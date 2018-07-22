use errors::{Result, Error};

use tun_tap::Iface;
use tun_tap::Mode::Tun;

use std::sync::Arc;
use std::net::Ipv4Addr;

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
    pub fn welcome(addr: Ipv4Addr) -> Hello {
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
    pub addr: Ipv4Addr,
}

pub fn open_tun(tun: &str)-> Result<(Arc<Iface>, Arc<Iface>)> {
    let iface = Iface::new(&tun, Tun)?;
    info!("opened tun device: {:?}", iface.name());
    // TODO: ip link set up dev iface.name()


    let tx = Arc::new(iface);
    let rx = tx.clone();
    Ok((tx, rx))
}
