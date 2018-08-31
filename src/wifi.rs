use dhcp;
use std::net::IpAddr;
use errors::*;

use trust_dns_resolver;


#[derive(Debug)]
pub struct Network {
    pub ap: String,
    pub essid: String,
    pub encryption: String,
    pub quality: String,
    pub signal: i32,
    pub channel: u16,
    pub mode: String,
}

impl Network {
    pub fn build(ap: &mut Option<String>,
                 essid: &mut Option<String>,
                 encryption: &mut Option<String>,
                 quality: &mut Option<String>,
                 signal: &mut Option<i32>,
                 channel: &mut Option<u16>,
                 mode: &mut Option<String>) -> Self {
        Network {
            ap: ap.take().unwrap_or(String::new()),
            essid: essid.take().unwrap_or(String::new()),
            encryption: encryption.take().unwrap_or(String::new()),
            quality: quality.take().unwrap_or(String::new()),
            signal: signal.take().unwrap_or(0),
            channel: channel.take().unwrap_or(0),
            mode: mode.take().unwrap_or(String::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub ssid: Option<String>,
    pub router: String,
    pub dns: Vec<IpAddr>,

    pub has_uplink: Option<bool>,
    pub script_used: Option<String>,
}

impl NetworkStatus {
    pub fn new(ssid: Option<String>, config: dhcp::NetworkConfig) -> NetworkStatus {
        NetworkStatus {
            ssid: ssid,
            router: config.routers,
            dns: config.dns_servers,

            has_uplink: None,
            script_used: None,
        }
    }

    pub fn empty() -> NetworkStatus {
        NetworkStatus {
            ssid: None,
            router: String::new(),
            dns: vec![],

            has_uplink: None,
            script_used: None,
        }
    }

    pub fn from_system() -> Result<NetworkStatus> {
        let (config, _opts) = trust_dns_resolver::system_conf::read_system_conf()?;
        let dns = config.name_servers().iter()
            .map(|x| x.socket_addr.ip().clone())
            .collect();

        Ok(NetworkStatus {
            ssid: None,
            router: String::new(),
            dns: dns,

            has_uplink: None,
            script_used: None,
        })
    }

    pub fn set_uplink_status(&mut self, uplink: Option<bool>) {
        self.has_uplink = uplink;
    }
}
