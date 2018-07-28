use errors::{Result, Error, ResultExt};

use args::snaild::Vpnd;
use config::{Config, VpnServerConfig};
use vpn::{self, Hello};
use vpn::crypto::{Handshake, Channel};
use vpn::transport::ServerTransport;
use vpn::transport::udp::UdpServer;
use vpn::wire::Packet;

use base64;
use cidr::{Inet, Ipv4Inet};
use serde_json;
use tun_tap::Iface;
use pktparse::{ipv4};
use rand::{thread_rng, Rng};

use std::thread;
use std::result;
use std::time::{Duration, Instant};
use std::sync::{mpsc, Arc};
use std::sync::mpsc::RecvTimeoutError;
use std::collections::HashSet;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};


#[derive(Debug)]
pub struct Lease {
    session: Channel,
    addr: Ipv4Addr,
}

impl Lease {
    #[inline]
    pub fn new(session: Channel, addr: Ipv4Addr) -> Lease {
        Lease {
            session,
            addr,
        }
    }

    #[inline]
    pub fn remote_pubkey(&self) -> Result<Vec<u8>> {
        self.session.remote_pubkey()
    }

    #[inline]
    pub fn decrypt(&mut self, packet: &Packet) -> Result<Vec<u8>> {
        self.session.decrypt(packet)
    }

    #[inline]
    pub fn encrypt(&mut self, msg: &[u8]) -> Result<Packet> {
        self.session.encrypt(msg)
    }
}

#[derive(Debug)]
pub enum Session {
    Handshake(Handshake),
    Channel(Lease),
}

#[derive(Debug)]
pub struct Server {
    socket: Arc<UdpServer>,
    tun: Arc<Iface>,
    addr: Ipv4Inet,
    timeout: Duration,

    clients: HashMap<SocketAddr, Session>, // TODO: this needs to reference self.leases
    leases: HashMap<Ipv4Addr, SocketAddr>,
    timeouts: HashMap<SocketAddr, Instant>,

    pool_start: Ipv4Addr,
    pool_end: Ipv4Addr,
    authorized: HashSet<Vec<u8>>,
    privkey: Vec<u8>,
}

impl Server {
    pub fn new(socket: Arc<UdpServer>,
               tun: Arc<Iface>,
               addr: Ipv4Inet,
               timeout: Duration,

               server_privkey: &str,
               pool_start: Ipv4Addr,
               pool_end: Ipv4Addr,
               authorized: &[String]) -> Result<Server> {
        let server_privkey = base64::decode(&server_privkey)?;

        let authorized = authorized.iter()
            .map(|key| base64::decode(&key))
            .collect::<result::Result<HashSet<_>, _>>()?;

        Ok(Server {
            socket,
            tun,
            addr,
            timeout,

            clients: HashMap::new(),
            leases: HashMap::new(),
            timeouts: HashMap::new(),

            pool_start,
            pool_end,
            authorized,
            privkey: server_privkey,
        })
    }

    #[inline]
    fn network_send(&self, pkt: &Packet, dest: &SocketAddr) -> Result<()> {
        self.socket.send_to(pkt, dest)
    }

    #[inline]
    fn tun_send(&self, pkt: &[u8]) -> Result<usize> {
        self.tun.send(pkt)
            .context("failed to write to tun device")
            .map_err(Error::from)
    }

    #[inline]
    pub fn is_authorized(&self, pubkey: &[u8]) -> bool {
        self.authorized.contains(pubkey)
    }

    pub fn allocate_ip(&mut self, remote: &SocketAddr) -> Result<Ipv4Addr> {
        let start = u32::from(self.pool_start);
        let end = u32::from(self.pool_end);

        // check if we have spare IPs left
        if self.leases.len() >= (end - start) as usize {
            bail!("no spare ips left in pool")
        }

        let mut rng = thread_rng();
        loop {
            let addr: u32 = rng.gen_range(start, end);
            let addr = Ipv4Addr::from(addr);

            // check if IP is available
            if !self.leases.contains_key(&addr) {
                self.leases.insert(addr.clone(), remote.clone());
                return Ok(addr);
            }
        }
    }

    pub fn setup_channel(&mut self, src: &SocketAddr, responder: &Channel) -> Result<Ipv4Inet> {
        let remote_key = responder.remote_pubkey()?;
        if !self.is_authorized(&remote_key) {
            bail!("client not authorized");
        }

        info!("[{}] client successfully authorized", src);
        // TODO: establishing a new channel should kill all old sessions of that pubkey

        let addr = self.allocate_ip(&src)?;
        info!("[{}] assigning ip to client: {}", src, addr);

        let addr = Ipv4Inet::new(addr, self.addr.network_length())?;
        Ok(addr)
    }

    #[inline]
    fn insert_handshake(&mut self, src: &SocketAddr, responder: Handshake) {
        self.clients.insert(src.clone(), Session::Handshake(responder));
        self.timeouts.insert(src.clone(), Instant::now());
    }

    #[inline]
    fn insert_channel(&mut self, src: &SocketAddr, responder: Lease) {
        self.clients.insert(src.clone(), Session::Channel(responder));
        self.timeouts.insert(src.clone(), Instant::now());
    }

    pub fn network_insert(&mut self, src: &SocketAddr, pkt: &Packet) -> Result<()> {
        match self.clients.remove(src) {
            Some(Session::Channel(mut channel)) => {
                let mut msg = channel.decrypt(pkt)?;
                // TODO: verify src IP
                self.tun_send(&msg)?;

                let pkt = channel.encrypt(&msg)?;
                self.network_send(&pkt, src)?;

                self.insert_channel(src, channel);
            },
            Some(Session::Handshake(mut handshake)) => {
                let pkt = pkt.handshake()?;
                handshake.insert(&pkt.bytes)?;

                if handshake.is_handshake_finished() {
                    info!("[{}] switching into transport mode", src);
                    let mut responder = handshake.channel()?;

                    match self.setup_channel(src, &responder) {
                        Ok(addr) => {
                            let welcome = serde_json::to_string(&Hello::welcome(addr.clone(),
                                                                                self.addr.address()))?;
                            let pkt = responder.encrypt(welcome.as_bytes())?;

                            self.network_send(&pkt, &src)?;
                            self.insert_channel(src, Lease::new(responder, addr.address()));
                        },
                        Err(e) => {
                            warn!("[{}] client rejected: {:?}", src, e);
                            let rejected = serde_json::to_string(&Hello::reject(e))?;
                            let pkt = responder.encrypt(rejected.as_bytes())?;

                            self.network_send(&pkt, &src)?;
                        },
                    }
                } else {
                    let pkt = handshake.take_packet()?;
                    self.insert_handshake(src, handshake);
                    self.network_send(&pkt, src)?;
                }
            },
            None => {
                let pkt = pkt.handshake()?;
                let mut responder = Handshake::responder(&self.privkey)?;
                responder.insert(&pkt.bytes)?;

                let pkt = responder.take_packet()?;
                self.insert_handshake(src, responder);
                self.network_send(&pkt, src)?;
            },
        }

        Ok(())
    }

    pub fn tun_insert(&mut self, dest: &Ipv4Addr, pkt: &[u8]) -> Result<()> {
        // TODO
        let client = match self.leases.get(dest) {
            Some(client) => client,
            None => bail!("ip has no active lease"),
        };

        // TODO: removing and re-inserting the channel is ugly
        let mut channel = self.clients.remove(client).unwrap();

        // TODO: verify sender IP

        // TODO: this is always true
        if let Session::Channel(ref mut channel) = channel {
            let pkt = channel.encrypt(pkt)?;
            self.network_send(&pkt, &client)?;
        }

        self.clients.insert(*client, channel);

        Ok(())
    }

    pub fn disconnect_timeout_clients(&mut self) -> Result<()> { // TODO: this function shouldn't fail
        if self.clients.is_empty() {
            return Ok(());
        }

        let mut dead = Vec::new();
        let now = Instant::now();

        for (src, last_packet) in &self.timeouts {
            if now.duration_since(*last_packet) > self.timeout {
                dead.push(src.clone());
            }
        }

        for dead in dead {
            info!("client timeout: {:?}", dead);
            match self.clients.remove(&dead) {
                Some(Session::Channel(lease)) => {
                    self.leases.remove(&lease.addr);
                },
                _ => (),
            }
            self.timeouts.remove(&dead);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Event {
    Tun((Ipv4Addr, Vec<u8>)),
    Udp((Packet, SocketAddr)),
}

pub fn tun_thread(tx: mpsc::Sender<Event>, tun: Arc<Iface>) -> Result<()> {
    let mut buffer = vec![0; 1504]; // MTU + 4 for the header

    loop {
        let n = tun.recv(&mut buffer)?;

        let pkt = &buffer[4..n];
        debug!("recv(tun): {:?}", pkt);

        if let Ok((_remaining, ipv4)) = ipv4::parse_ipv4_header(&pkt) {
            if ipv4.version != 4 {
                continue;
            }

            debug!("recv(ipv4): {:?}", ipv4);
            tx.send(Event::Tun((ipv4.dest_addr.into(), buffer[..n].to_vec()))).unwrap();
        }
    }
}

pub fn udp_thread(tx: mpsc::Sender<Event>, socket: Arc<UdpServer>) -> Result<()> {
    loop {
        let (msg, src) = socket.recv_from()?;
        debug!("recv(udp): {} -> {:?}", src, msg);
        tx.send(Event::Udp((msg, src))).unwrap();
    }
}

pub fn vpn_thread(rx: mpsc::Receiver<Event>,
                  socket: Arc<UdpServer>,
                  tun: Arc<Iface>,
                  vpn_config: &VpnServerConfig) -> Result<()> {

    let timeout = Duration::from_secs(vpn_config.ping_timeout.unwrap_or(60));

    let mut server = Server::new(socket, tun,
                                 vpn_config.gateway_ip.clone(),
                                 timeout.clone(),

                                 &vpn_config.server_privkey,
                                 vpn_config.pool_start.clone(),
                                 vpn_config.pool_end.clone(),
                                 &vpn_config.clients)?;

    loop {
        match rx.recv_timeout(timeout) {
            Ok(Event::Udp((msg, src))) => if let Err(e) = server.network_insert(&src, &msg) {
                warn!("[{}] error: {:?}", src, e);
            },
            Ok(Event::Tun((dest, pkt))) => if let Err(e) = server.tun_insert(&dest, &pkt) {
                warn!("[{}] error: {:?}", dest, e);
            },
            Err(RecvTimeoutError::Timeout) => (),
            Err(RecvTimeoutError::Disconnected) => break,
        };

        server.disconnect_timeout_clients()?;
    }

    Ok(())
}

pub fn run(args: Vpnd, config: &Config) -> Result<()> {
    let vpn_config = config.vpn.as_ref()
        .ok_or(format_err!("vpn not configured"))?.server.as_ref()
        .ok_or(format_err!("vpn server not configured"))?;

    let tun = Arc::new(vpn::open_tun(&args.interface)?);

    vpn::ipconfig(&args.interface,
                  &vpn_config.gateway_ip)?;

    let socket = Arc::new(UdpServer::bind(&vpn_config.bind)?);
    let (tx, rx) = mpsc::channel();

    let t1 = {
        let tx = tx.clone();
        let tun = tun.clone();
        thread::spawn(move || {
            tun_thread(tx, tun)
                .expect("tun rx thread failed");
        })
    };

    let t2 = {
        let socket = socket.clone();
        thread::spawn(move || {
            udp_thread(tx, socket)
                .expect("udp rx thread failed");
        })
    };

    let t3 = {
        let vpn_config = vpn_config.to_owned();
        thread::spawn(move || {
            vpn_thread(rx, socket, tun, &vpn_config)
                .expect("vpn thread failed");
        })
    };

    // TODO: timer thread to remove dead clients
    // TODO: ^ could be implemented using a recv timeout on mpsc

    for t in vec![t1, t2, t3] {
        t.join()
            .map_err(|_| format_err!("thread failed"))?;
    }

    Ok(())
}
