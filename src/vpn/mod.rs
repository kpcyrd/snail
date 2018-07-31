use errors::{Result, Error, ResultExt};
use utils;

use cidr::Ipv4Inet;
use tun_tap::Iface;
use tun_tap::Mode::Tun;

use std::net::Ipv4Addr;
use std::time::Duration;

pub mod crypto;
pub mod client;
pub mod ifconfig;
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
    pub fn welcome(addr: Ipv4Inet, gateway: Ipv4Addr, timeout: Duration) -> Hello {
        Hello::Welcome(HelloSettings {
            addr,
            gateway,
            timeout,
        })
    }

    pub fn reject(err: Error) -> Hello {
        Hello::Rejected(err.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloSettings {
    pub addr: Ipv4Inet,
    pub gateway: Ipv4Addr,
    pub timeout: Duration,
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

pub fn add_route(range: &Ipv4Inet, gateway: &Ipv4Addr) -> Result<()> {
    utils::cmd("ip", &["route", "del", &range.to_string(), "via", &gateway.to_string()]).ok();
    utils::cmd("ip", &["route", "add", &range.to_string(), "via", &gateway.to_string()])
        .context("failed to set route")?;
    Ok(())
}

pub fn get_route(target: &Ipv4Addr) -> Result<Ipv4Addr> {
    use regex::Regex;

    let output = utils::cmd("ip", &["route", "get", &target.to_string()])
        .context("failed to get route from table")?;

    let re = Regex::new(r#"^[\d\.]+ via ([\d\.]+)"#).unwrap();

    if let Some(gateway) = re.captures(&output) {
        let gateway = gateway.get(1).unwrap();
        Ok(gateway.as_str().parse()?)
    } else {
        bail!("no gateway found");
    }
}

pub fn tunnel_all_traffic(gateway: &Ipv4Addr) -> Result<()> {
    add_route(&"0.0.0.0/1".parse()?, &gateway)?;
    add_route(&"128.0.0.0/1".parse()?, &gateway)?;
    Ok(())
}
