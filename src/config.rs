use toml;

use errors::Result;
use ipc;

use std::fs;
use std::collections::HashMap;


pub const PATH: &str = "/etc/snail.conf";

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub daemon: DaemonConfig,
    #[serde(default)]
    pub scripts: ScriptConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DaemonConfig {
    #[serde(default="default_socket")]
    pub socket: String,
    pub socket_group: Option<String>,
}

impl Default for DaemonConfig {
    fn default() -> DaemonConfig {
        DaemonConfig {
            socket: default_socket(),
            socket_group: None,
        }
    }
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
