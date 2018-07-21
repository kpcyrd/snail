use errors::{Result};

use tun_tap::Iface;
use tun_tap::Mode::Tun;

use std::sync::Arc;

pub mod crypto;
pub mod client;
pub mod server;
pub mod transport;


pub fn open_tun(tun: &str)-> Result<(Arc<Iface>, Arc<Iface>)> {
    let iface = Iface::new(&tun, Tun)?;
    info!("opened tun device: {:?}", iface.name());
    // TODO: ip link set up dev iface.name()


    let tx = Arc::new(iface);
    let rx = tx.clone();
    Ok((tx, rx))
}
