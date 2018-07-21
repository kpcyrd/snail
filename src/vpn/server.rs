use errors::Result;

use args::snaild::Vpnd;
use config::Config;
use vpn;
use vpn::crypto::ServerHandshake;
use vpn::transport::udp::UdpServer;

use base64;
use tun_tap::Iface;
use pktparse::{ipv4};

use std::thread;
use std::result;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;


#[derive(Debug, Clone)]
pub struct State {
    pub foo: Arc<Mutex<String>>,
}

impl State {
    pub fn new() -> State {
        State {
            foo: Arc::new(Mutex::new(String::from("TODO"))),
        }
    }
}

pub fn udp_thread(_state: State, config: &Config) -> Result<()> {
    let vpn_config = config.vpn.as_ref()
        .ok_or(format_err!("vpn not configured"))?.server.as_ref()
        .ok_or(format_err!("vpn client not configured"))?;

    let clients = vpn_config.clients.iter()
        .map(|key| base64::decode(&key))
        .collect::<result::Result<HashSet<_>, _>>()?;

    let socket = UdpServer::bind("127.0.0.1:7788")?; // TODO
    let mut responder = ServerHandshake::responder(socket, &vpn_config.server_privkey)?;

    let responder2;

    // TODO: this should be one loop
    loop {
        let src = match responder.recv_from() {
            Ok(src) => src,
            Err(_) => continue,
        };

        if responder.is_handshake_finished() {
            info!("[{}] switching into transport mode", src); // TODO: src missing
            let mut responder = responder.channel()?;

            let remote_key = responder.remote_pubkey()?;
            if clients.contains(&remote_key) {
                info!("[{}] client successfully authorized", src);
                responder.send_to(b"welcome", &src)?;
            } else {
                responder.send_to(b"rejected", &src)?;
                warn!("[{}] client rejected", src);
                bail!("client not authorized");
            }

            responder2 = responder;
            break;
        } else {
            responder.send_to(&src)?;
        }
    }

    let mut responder = responder2;

    loop {
        let (mut msg, src) = responder.recv_from()?;
        println!("{:?}", String::from_utf8(msg.clone()));
        msg.reverse();
        responder.send_to(&msg, &src)?;
    }

    // Ok(())
}

pub fn rx_thread(tun_rx: Arc<Iface>) -> Result<()> {
    let mut buffer = vec![0; 1504]; // MTU + 4 for the header

    loop {
        let n = tun_rx.recv(&mut buffer)?;

        let pkt = &buffer[4..n];
        debug!("recv(tun): {:?}", pkt);

        if let Ok((_remaining, ipv4)) = ipv4::parse_ipv4_header(&pkt) {
            if ipv4.version != 4 {
                continue;
            }

            info!("recv(ipv4): {:?}", ipv4);
            warn!("todo: forward packet to {:?}", ipv4.dest_addr);
        }
    }

    // Ok(())
}

pub fn tx_thread(_tx: Arc<Iface>) -> Result<()> {

/*
let mut buffer = vec![0; 1504]; // MTU + 4 for the header
loop {
let n = tap.recv(&mut buffer)?;
debug!("recv(tap): {:?}", &buffer[4..n]);
*/

    // unimplemented!()
    Ok(())
}

pub fn run(args: Vpnd, config: &Config) -> Result<()> {
    let state = State::new();

    let t1 = {
        let state = state.clone();
        let config = config.to_owned();
        thread::spawn(move || {
            udp_thread(state, &config)
                .expect("vpn udp thread failed");
        })
    };

    let (tx, rx) = vpn::open_tun(&args.interface)?;

    let t2 = thread::spawn(move || {
        rx_thread(rx)
            .expect("vpn rx thread failed");
    });

    let t3 = thread::spawn(move || {
        tx_thread(tx)
            .expect("vpn tx thread failed");
    });

    // TODO: timer thread to remove dead clients

    for t in vec![t1, t2, t3] {
        t.join()
            .map_err(|_| format_err!("thread failed"))?;
    }

    Ok(())
}
