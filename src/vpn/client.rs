use errors::{Result, Error, ResultExt};

use args::snaild::Vpn;
use config::{Config, VpnClientConfig};
use vpn::{self, Hello, Iface};
use vpn::crypto::{Session, Handshake};
use vpn::transport::ClientTransport;
use vpn::transport::udp::UdpClient;
use vpn::wire::Packet;

use std::thread;
use std::net::SocketAddr;
use std::sync::{mpsc, Arc};
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use base64;
use cidr::{Ipv4Inet, Inet};
use serde_json;


#[derive(Debug)]
pub struct Client {
    session: Option<Session>,
    socket: Arc<UdpClient>,
    tun: Arc<Iface>,
    pending_hello: bool,

    interface: String,
    remote: SocketAddr,
    tunnel_all_traffic: bool,

    last_client_ping: Option<Instant>,
    last_server_ping: Option<Instant>,
    ping_interval: Duration,
}

impl Client {
    pub fn new(socket: Arc<UdpClient>,
               tun: Arc<Iface>,

               interface: String,
               remote: SocketAddr,
               tunnel_all_traffic: bool,

               server_pubkey: &str,
               client_privkey: &str) -> Result<Client> {
        let server_pubkey = base64::decode(&server_pubkey)?;
        let client_privkey = base64::decode(&client_privkey)?;

        // TODO: this should move to start_session
        let handshake = Handshake::initiator(&server_pubkey, &client_privkey)?;

        Ok(Client {
            session: Some(Session::Handshake(handshake)),
            socket,
            tun,

            interface,
            remote,
            tunnel_all_traffic,

            pending_hello: false,
            last_client_ping: None,
            last_server_ping: None,
            ping_interval: Duration::from_secs(30),
        })
    }

    #[inline]
    fn network_send(&mut self, pkt: &Packet) -> Result<()> {
        self.last_client_ping = Some(Instant::now());
        self.socket.send(pkt)
    }

    #[inline]
    fn tun_send(&self, pkt: &[u8]) -> Result<usize> {
        self.tun.send(pkt)
            .map_err(Error::from)
    }

    pub fn start_session(&mut self) -> Result<()> {
        let mut session = self.session.take().unwrap();

        if let Session::Handshake(ref mut session) = session {
            let msg = session.take()?;
            let pkt = Packet::make_handshake(msg);
            self.network_send(&pkt)?;
        }

        self.session = Some(session);
        Ok(())
    }

    pub fn network_insert(&mut self, pkt: &Packet) -> Result<()> {
        self.last_server_ping = Some(Instant::now());
        let session = self.session.take().unwrap();

        let session = match session {
            Session::Handshake(mut session) => {
                // take packet
                let pkt = pkt.handshake()?;
                session.insert(&pkt.bytes)?;

                // send reply
                let msg = session.take()?;
                let pkt = Packet::make_handshake(msg);
                self.network_send(&pkt)?;

                if session.is_handshake_finished() {
                    info!("switching into transport mode");
                    let session = session.channel()?;
                    self.pending_hello = true;
                    Session::Channel(session)
                } else {
                    // handshake still in progress
                    Session::Handshake(session)
                }
            },
            Session::Channel(mut session) => {
                let pkt = pkt.transport()?;
                let msg = match session.decrypt(pkt) {
                    Ok(pkt) => pkt,
                    Err(err) => {
                        // TODO: reinserting is ugly
                        self.session = Some(Session::Channel(session));

                        return Err(err);
                    },
                };

                if self.pending_hello {
                    self.pending_hello = false;

                    let msg = serde_json::from_slice::<Hello>(&msg)?;
                    debug!("server said: {:?}", msg);
                    match msg {
                        Hello::Welcome(settings) => {
                            info!("server accepted session and sent settings: {:?}", settings);

                            vpn::ipconfig(&self.interface,
                                          &settings.addr)?;

                            if self.tunnel_all_traffic {
                                info!("redirecting all traffic");

                                // add route for vpn server ip first
                                if let SocketAddr::V4(ref remote) = self.remote {
                                    let old_gateway = vpn::get_route(remote.ip())?;
                                    let route = Ipv4Inet::new(remote.ip().clone(), 32)?;
                                    vpn::add_route(&route, &old_gateway)?;
                                }

                                vpn::tunnel_all_traffic(&settings.gateway)?;
                            }
                        },
                        Hello::Rejected(err) => {
                            bail!("server rejected us: {:?}", err);
                        },
                    }
                } else {
                    if !msg.is_empty() {
                        self.tun_send(&msg)
                            .context("failed to write to tun device")?;
                    }
                }
                Session::Channel(session)
            },
        };

        // TODO: reinserting is ugly
        self.session = Some(session);
        Ok(())
    }

    pub fn tun_insert(&mut self, msg: &[u8]) -> Result<()> {
        let mut session = self.session.take().unwrap();

        if let Session::Channel(ref mut session) = session {
            if !self.pending_hello {
                let pkt = session.encrypt(&msg)?;
                self.network_send(&pkt)?;
            }
        };

        // TODO: reinserting is ugly
        self.session = Some(session);
        Ok(())
    }

    pub fn send_ping(&mut self) -> Result<()> {
        if let Some(Session::Channel(mut session)) = self.session.take() {
            debug!("sending ping");
            let pkt = session.encrypt(&[])?;
            self.network_send(&pkt)?;

            // TODO: reinserting is ugly
            self.session = Some(Session::Channel(session));
        }
        Ok(())
    }

    #[inline]
    pub fn timeout(&self) -> Duration {
        self.ping_interval.clone() // TODO: inline maybe
    }

    pub fn ping_if_needed(&mut self) -> Result<()> {
        let now = Instant::now();

        if let Some(last_client_ping) = self.last_client_ping.clone() {
            if now.duration_since(last_client_ping) > self.ping_interval {
                self.send_ping()?;
            }
        }

        if let Some(last_server_ping) = self.last_server_ping.clone() {
            if now.duration_since(last_server_ping) > self.ping_interval * 2 {
                bail!("server doesn't respond");
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum Event {
    Tun(Vec<u8>),
    Udp(Packet),
}

pub fn tun_thread(tx: mpsc::Sender<Event>, tun: Arc<Iface>) -> Result<()> {
    let mut buffer = vec![0; 1504]; // MTU + 4 for the header

    loop {
        let n = tun.recv(&mut buffer)?;

        let pkt = &buffer[4..n];
        debug!("recv(tun): {:?}", pkt);

        // TODO: only send ipv4 traffic
        tx.send(Event::Tun(buffer[..n].to_vec())).unwrap();
    }
}

pub fn udp_thread(tx: mpsc::Sender<Event>, socket: Arc<UdpClient>) -> Result<()> {
    loop {
        let msg = socket.recv()?;
        debug!("recv(udp): {:?}", msg);
        tx.send(Event::Udp(msg)).unwrap();
    }
}

pub fn vpn_thread(rx: mpsc::Receiver<Event>,
                  socket: Arc<UdpClient>,
                  tun: Arc<Iface>,
                  interface: String,
                  vpn_config: &VpnClientConfig) -> Result<()> {
    let mut client = Client::new(socket,
                                 tun,

                                 interface,
                                 vpn_config.remote,
                                 vpn_config.tunnel_all_traffic,

                                 &vpn_config.server_pubkey,
                                 &vpn_config.client_privkey)?;

    info!("starting vpn session");
    client.start_session()?;

    loop {
        match rx.recv_timeout(client.timeout()) {
            Ok(Event::Udp(msg)) => if let Err(e) = client.network_insert(&msg) {
                warn!("[udp] error: {:?}", e);
            },
            Ok(Event::Tun(pkt)) => if let Err(e) = client.tun_insert(&pkt) {
                warn!("[tun] error: {:?}", e);
            },
            Err(RecvTimeoutError::Timeout) => (),
            Err(RecvTimeoutError::Disconnected) => break,
        }

        client.ping_if_needed()?;
    }

    Ok(())
}

pub fn run(args: Vpn, config: &Config) -> Result<()> {
    let vpn_config = config.vpn.as_ref()
        .ok_or(format_err!("vpn not configured"))?.client.as_ref()
        .ok_or(format_err!("vpn client not configured"))?;

    let tun = Arc::new(vpn::open_tun(&args.interface)?);
    let socket = Arc::new(UdpClient::connect(&vpn_config.remote)?);
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
            vpn_thread(rx, socket, tun, args.interface, &vpn_config)
                .expect("vpn thread failed");
        })
    };

    for t in vec![t1, t2, t3] {
        t.join()
            .map_err(|_| format_err!("thread failed"))?;
    }

    Ok(())
}
