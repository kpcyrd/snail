use errors::Result;

use args::snaild::Vpn;
use config::Config;
use vpn::crypto::Handshake;

use base64;

use std::net::UdpSocket;


pub fn run(_args: Vpn, config: &Config) -> Result<()> {
    let vpn_config = config.vpn.as_ref()
        .ok_or(format_err!("vpn not configured"))?.client.as_ref()
        .ok_or(format_err!("vpn client not configured"))?;

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.connect("127.0.0.1:7788")?; // TODO

    let mut stage = 0;

    let server_pubkey = base64::decode(&vpn_config.server_pubkey)?;
    let client_privkey = base64::decode(&vpn_config.client_privkey)?;

    let mut initiator = Handshake::initiator(&server_pubkey, &client_privkey).unwrap();

    let mut buf = [0; 1600]; // TODO: adjust size

    loop {
        info!("handshake stage {}", stage);
        let msg = initiator.take()?;
        socket.send(&msg)?;

        stage += 1;

        if stage == 2 {
            break;
        } else {
            let amt = socket.recv(&mut buf)?;
            let buf = &mut buf[..amt];

            initiator.insert(&buf)?;
        }
    }

    info!("switching into transport mode");
    let mut initiator = initiator.transport()?;

    {
        let amt = socket.recv(&mut buf)?;
        let buf = &mut buf[..amt];
        let msg = initiator.decrypt(&buf)?;
        info!("server said: {:?}", String::from_utf8(msg)?);
    }

    use std::io::{self, Read};

    let mut stdin = io::stdin();
    loop {
        let mut asdf = [0u8; 400];
        let amt = stdin.read(&mut asdf)?;

        info!("sending encrypted data");
        let msg = initiator.encrypt(&asdf[..amt])?;
        socket.send(&msg)?;

        let amt = socket.recv(&mut buf)?;
        let buf = &mut buf[..amt];

        let msg = initiator.decrypt(&buf)?;

        println!("{:?}", String::from_utf8(msg)?);
    }

    // Ok(())
}
