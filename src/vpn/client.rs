use errors::Result;

use args::snaild::Vpn;
use config::Config;
use vpn::Hello;
use vpn::crypto::ClientHandshake;
use vpn::transport::udp::UdpClient;

use serde_json;


pub fn udp_thread(_args: Vpn, config: &Config) -> Result<()> {
    let vpn_config = config.vpn.as_ref()
        .ok_or(format_err!("vpn not configured"))?.client.as_ref()
        .ok_or(format_err!("vpn client not configured"))?;

    let socket = UdpClient::connect("127.0.0.1:0", "127.0.0.1:7788")?; // TODO: remote is hardcoded
    let mut initiator = ClientHandshake::initiator(socket, &vpn_config.server_pubkey, &vpn_config.client_privkey)?;

    loop {
        initiator.send()?;

        if initiator.is_handshake_finished() {
            break;
        }

        initiator.recv()?;
    }

    info!("switching into transport mode");
    let mut initiator = initiator.channel()?;

    let msg = initiator.recv()?;

    let msg = serde_json::from_slice::<Hello>(&msg)?;
    debug!("server said: {:?}", msg);
    match msg {
        Hello::Welcome(settings) => {
            info!("server accepted session and sent settings: {:?}", settings);
        },
        Hello::Rejected(err) => {
            bail!("server rejected us: {:?}", err);
        },
    }

    use std::io::{self, Read};

    let mut stdin = io::stdin();
    loop {
        let mut asdf = [0u8; 400];
        let amt = stdin.read(&mut asdf)?;

        if amt == 0 {
            break;
        }

        info!("sending encrypted data");
        initiator.send(&asdf[..amt])?;

        let msg = initiator.recv()?;
        println!("{:?}", String::from_utf8(msg)?);
    }

    Ok(())
}

pub fn run(_args: Vpn, config: &Config) -> Result<()> {
    udp_thread(_args, config)
}
