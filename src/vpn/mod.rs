use errors::{Result, Error, ResultExt};
use utils;

use cidr::Ipv4Inet;
use tun_tap::Iface;
use tun_tap::Mode::Tun;

pub mod crypto;
pub mod client;
pub mod server;
pub mod transport;
pub mod wire;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Hello {
    Welcome(HelloSettings),
    Rejected(String),
}

impl Hello {
    pub fn welcome(addr: Ipv4Inet) -> Hello {
        Hello::Welcome(HelloSettings {
            addr,
        })
    }

    pub fn reject(err: Error) -> Hello {
        Hello::Rejected(err.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloSettings {
    pub addr: Ipv4Inet,
}

pub fn open_tun(tun: &str)-> Result<Iface> {
    let iface = Iface::new(&tun, Tun)?;
    info!("opened tun device: {:?}", iface.name());
    // TODO: ip link set up dev iface.name()

    Ok(iface)
}

pub fn ipconfig(interface: &str, addr: &Ipv4Inet) -> Result<()> {
    utils::cmd("ip", &["link", "set", &interface, "up"])
        .context("failed to set tun device up")?;

    info!("configuring tun device: {}", addr);
    utils::cmd("ip", &["address",
                       "add", &addr.to_string(),
                       "dev", &interface])
        .context("failed to set ip on tun device")?;

    Ok(())
}
