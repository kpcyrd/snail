use errors::Result;

use args::snaild::Vpnd;
use vpn;

use tun_tap::Iface;
use pktparse::{ipv4};

use std::thread;
use std::sync::{Arc, Mutex};
use std::net::UdpSocket;


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

pub fn udp_thread(_state: State) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:7788")?; // TODO

    let mut stage = 0;

    use snow::NoiseBuilder;
    use snow::params::NoiseParams;

    // TODO: consider Noise_XXpsk3_25519_ChaChaPoly_BLAKE2s
    let params: NoiseParams = "Noise_XK_25519_ChaChaPoly_BLAKE2s".parse().unwrap();

    let builder: NoiseBuilder = NoiseBuilder::new(params.clone());
    let static_key = builder.generate_private_key().unwrap();

    let mut noise = builder
        .local_private_key(&static_key)
        .psk(3, b"fidget spinner fucking rule")
        .build_responder().unwrap();

    let mut buf = [0; 1600]; // TODO: adjust size
    let mut buf2 = [0; 1600]; // TODO: rename, this is used for noise
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;

        let buf = &mut buf[..amt];
        info!("recv(udp, {}): {:?}", src, buf);

        if stage == 0 {
            // <- e
            noise.read_message(&buf, &mut buf2).unwrap();

            // -> e, ee, s, es
            let len = noise.write_message(&[0u8; 0], &mut buf2).unwrap();
            socket.send_to(&buf2[..len], &src)?;

            stage = 1;
        } else if stage == 1 {
            // <- s, se
            noise.read_message(&buf, &mut buf2).unwrap();

            // Transition the state machine into transport mode now that the handshake is complete.
            // let mut _noise = noise.into_transport_mode().unwrap();
        }


        // buf.reverse();
        // socket.send_to(buf, &src)?;
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

pub fn run(args: Vpnd) -> Result<()> {
    let state = State::new();

    let t1 = {
        let state = state.clone();
        thread::spawn(move || {
            udp_thread(state)
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
