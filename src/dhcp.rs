use errors::Result;
/*
use std::net::SocketAddr;
use nix;
use nix::libc::SOL_SOCKET;
use nix::sys::socket::{AddressFamily, SockType, SockFlag, SockProtocol,
                       socket, setsockopt, sockopt, bind};


// TODO: get mac address of device


// fd = listen_socket(INADDR_ANY, CLIENT_PORT, client_config.interface);

pub fn listen_socket(addr: SocketAddr, interface: &str) -> Result<()> {
    let fd = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), SockProtocol::Udp)?;

    setsockopt(fd, sockopt::ReuseAddr, &true)?;
    setsockopt(fd, sockopt::Broadcast, &true)?;
    setsockopt(fd, sockopt::BINDTODEVICE(interface));

    // nix::sys::socket::bind(fd, &addr);

    // Ok(fd)
    unimplemented!()
}
*/
use std::env;
use std::process::Command;
use std::net::IpAddr;


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub ip_address: IpAddr,
    pub subnet_cidr: u8,
    pub network_number: String, // IPv4

    pub routers: String, // List of IPv4?
    pub dns_servers: Vec<IpAddr>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum UpdateMessage {
    Carrier,
    Bound(NetworkConfig),
    Reboot(NetworkConfig),
    Renew(NetworkConfig),
    NoCarrier,
    Stopped,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NetworkUpdate {
    pub interface: String,
    pub ssid: Option<String>,
    pub message: Option<UpdateMessage>,
    pub env: Vec<(String, String)>,
}

pub fn read_network_config() -> Result<NetworkConfig> {
    let ip_address = env::var("new_ip_address")?.parse()?;
    let subnet_cidr = env::var("new_subnet_cidr")?.parse()?;
    let network_number = env::var("new_network_number")?;

    let routers = env::var("new_routers")?;
    let dns_servers = env::var("new_domain_name_servers")?
                        .split(" ")
                        .map(|x| x.parse())
                        .collect::<::std::result::Result<_, _>>()?;

    Ok(NetworkConfig {
        ip_address,
        subnet_cidr,
        network_number,

        routers,
        dns_servers,
    })
}


pub fn read_dhcp_env() -> Result<NetworkUpdate> {
    let interface = env::var("interface")?;
    let ssid = match env::var("ifssid")?.as_str() {
        "" => None,
        ssid => Some(ssid.to_string()),
    };
    let env = env::vars().collect();

    Ok(NetworkUpdate {
        interface,
        ssid,
        message: match env::var("reason")?.as_str() {
            "CARRIER" => {
                // TODO: set interface up?
                Some(UpdateMessage::Carrier)
            },
            "BOUND" => {
                Some(UpdateMessage::Bound(read_network_config()?))
            },
            "REBOOT" => {
                // I only noticed those when running dhcpcd on an already configured interface
                Some(UpdateMessage::Reboot(read_network_config()?))
            },
            "RENEW" => {
                // we can probably ignore those
                Some(UpdateMessage::Renew(read_network_config()?))
            },
            "NOCARRIER" => {
                // the interface went down
                // TODO: set interface down?
                Some(UpdateMessage::NoCarrier)
            },
            "EXPIRE" => {
                // we couldn't renew our ip
                None
            },
            "PREINIT" => {
                // dhcpcd started, nothing to do
                None
            },
            "STOP" | "STOPPED" => {
                // dhcpcd stopped, nothing to do
                // TODO: set interface down?
                // TODO: we have to signal this so the event can be safely discarded
                Some(UpdateMessage::Stopped)
            },
            _ => None,
        },
        env,
    })
}


pub fn run_dhcpcd(conf: &str, interface: &str, hook: &str) -> Result<()> {

    // TODO: if hook is not absolute, we need to resolve this
    // let hook = env::current_exe().unwrap().to_str().unwrap(),

    // TODO: this should be replaced by a builtin dhcpcd eventually
    // XXX: maybe set -K, --nolink
    let mut child = Command::new("dhcpcd")
                    .args(&["-f", conf,
                            "-c", hook,
                            "-m", "2048", // ensure ethernet is prefered
                            "-B", // no background
                            interface])
                    .spawn()?;

    // wait until interface goes down
    // TODO: verify this returns when the interface goes down
    child.wait()?;

    // TODO: we can't get the info here because it is sent to the hook instead :/
    // TODO: might need snaild to relay to info for us

    Ok(())
}
