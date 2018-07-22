use toml;
use users;
use trust_dns::rr::LowerName;

use errors::Result;
use ipc;

use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::collections::HashMap;


pub const PATH: &str = "/etc/snail/snail.conf";

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    pub dns: Option<DnsConfig>,
    #[serde(default)]
    pub scripts: ScriptConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default="default_socket")]
    pub socket: String,
    pub socket_group: Option<String>,
    pub socket_gid: Option<(String, u32)>,
}

impl Default for DaemonConfig {
    fn default() -> DaemonConfig {
        DaemonConfig {
            socket: default_socket(),
            socket_group: None,
            socket_gid: None,
        }
    }
}

impl DaemonConfig {
    pub fn resolve_gid(&mut self) -> Result<()> {
        if let Some(group) = &self.socket_group {
            if let Some(gid) = users::get_group_by_name(&group) {
                self.socket_gid = Some((group.clone(), gid.gid()));
            } else {
                bail!("group not found");
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub user: Option<String>,
    #[serde(default)]
    pub strict_chroot: bool,

    /// this flag is going to be removed eventually
    #[serde(default)]
    pub danger_disable_seccomp_security: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DnsConfig {
    #[serde(default)]
    pub standalone: bool,

    pub bind: SocketAddr,

    pub servers: Vec<IpAddr>,
    pub port: u16,
    pub sni: String,

    #[serde(default)]
    pub records: HashMap<String, Vec<IpAddr>>,
    #[serde(default)]
    pub zones: HashMap<LowerName, Vec<IpAddr>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptConfig {
    pub paths: HashMap<String, ScriptFolder>,
    #[serde(default="default_agent")]
    pub user_agent: String,
}

impl Default for ScriptConfig {
    fn default() -> ScriptConfig {
        ScriptConfig {
            paths: HashMap::new(),
            user_agent: default_agent(),
        }
    }
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ScriptFolder {
}

fn default_socket() -> String {
    ipc::SOCKET.to_string()
}

fn default_agent() -> String {
    format!("snail/{}.{}", env!("CARGO_PKG_VERSION_MAJOR"),
                           env!("CARGO_PKG_VERSION_MINOR"))
}

pub fn read_from(path: &str) -> Result<Config> {
    let text = fs::read_to_string(path)?;
    load(&text)
}

#[inline]
pub fn load(text: &str) -> Result<Config> {
    let conf = toml::from_str(&text)?;
    Ok(conf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_empty() {
        let _config = load("").expect("failed to load config");
    }

    #[test]
    fn test_dns_config() {
        let _config = load(r#"
        [dns]
        bind = "127.0.0.1:53"

        servers = ["1.1.1.1",
                   "1.0.0.1",
                   "2606:4700:4700::1111",
                   "2606:4700:4700::1001"]
        port = 443
        sni = "cloudflare-dns.com"

        [dns.records]
        "foo.example.com" = ["192.0.2.10", "2001:DB8::10"]
        "bar.example.com" = ["192.0.2.20", "2001:DB8::20"]

        [dns.zones]
        "example.com" = ["192.0.2.2", "2001:DB8::2"]
        "corp.example.com" = ["192.0.2.3", "2001:DB8::3"]
        "#).expect("failed to load config");
    }

    #[test]
    fn test_script_paths() {
        let config = load(r#"
        [scripts.paths."/etc/foo"]
        [scripts.paths."/etc/bar"]
        "#).expect("failed to load config");
        println!("{:?}", config);

        let mut paths = HashMap::new();
        paths.insert("/etc/foo".into(), ScriptFolder { });
        paths.insert("/etc/bar".into(), ScriptFolder { });
        assert_eq!(config.scripts, ScriptConfig {
            paths,
            ..Default::default()
        });
    }
}
