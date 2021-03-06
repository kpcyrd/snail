use config::Config;
use dns::{Resolver, DnsResolver};
use errors::{Result, ResultExt};
use scripts::Script;
use web::{self, HttpClient};
use wifi::NetworkStatus;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::collections::HashMap;


#[derive(Debug, Clone)]
pub struct Loader {
    scripts: HashMap<String, (String, bool)>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            scripts: HashMap::new(),
        }
    }

    fn insert<I: Into<String>>(&mut self, name: I, script: String, private_script: bool) {
        self.scripts.insert(name.into(), (script, private_script));
    }

    pub fn init<C: HttpClient + 'static, R: DnsResolver + 'static>(&self, http: Arc<C>, resolver: Arc<R>) -> Result<Vec<Script<C, R>>> {
        self.scripts.iter()
            .map(|(_, (code, _))| {
                Script::load(code.to_string(), http.clone(), resolver.clone())
            })
            .collect()
    }

    pub fn init_default<I: Into<String>>(code: I) -> Result<Script<web::Client<Resolver>, Resolver>> {
        let resolver = Arc::new(Resolver::cloudflare());
        let http = Arc::new(web::Client::new(Resolver::cloudflare()));

        Script::load(code.into(), http, resolver)
    }

    pub fn init_from_status(&self, status: &NetworkStatus) -> Result<Vec<Script<web::Client<Resolver>, Resolver>>> {
        let resolver = Resolver::with_udp(&status.dns)?;
        let http = Arc::new(web::Client::new(resolver));
        let resolver = Arc::new(Resolver::with_udp(&status.dns)?);

        self.init(http, resolver)
    }

    pub fn init_all_scripts_default(config: &Config) -> Result<Vec<Script<web::Client<Resolver>, Resolver>>> {
        let mut loader = Loader::new();
        loader.load_all_scripts(config)?;

        let resolver = Arc::new(Resolver::cloudflare());
        let http = Arc::new(web::Client::new(Resolver::cloudflare()));

        loader.init(http, resolver)
    }

    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    pub fn load<I: Into<String>>(&mut self, name: I, code: String, private_script: bool) -> Result<()> {
        self.insert(name, code, private_script);
        Ok(())
    }

    pub fn load_all_scripts(&mut self, config: &Config) -> Result<usize> {
        let mut counter = 0;
        counter += self.load_default_scripts()?;
        counter += self.load_private_scripts(config)?;
        Ok(counter)
    }

    pub fn load_default_scripts(&mut self) -> Result<usize> {
        self.load_from_folder("/usr/lib/snaild/scripts", false)
    }

    pub fn load_private_scripts(&mut self, config: &Config) -> Result<usize> {
        let mut counter = 0;

        counter += self.load_from_folder("/etc/snail/scripts/", true)?;

        for (path, _) in &config.scripts.paths {
            counter += self.load_from_folder(path, true)?;
        }

        Ok(counter)
    }

    pub fn load_from_folder<P: AsRef<Path>>(&mut self, path: P, private_script: bool) -> Result<usize> {
        let mut counter = 0;

        if let Ok(paths) = fs::read_dir(&path) {
            for path in paths {
                let path = path?;

                let file_name = match path.file_name().into_string() {
                    Ok(file_name) => file_name,
                    Err(file_name) => {
                        warn!("invalid filename: {:?}", file_name);
                        continue
                    },
                };

                if !file_name.ends_with(".lua") {
                    continue;
                }

                let code = fs::read_to_string(path.path())
                    .context(format!("failed to open {:?}", path.path()))?;

                self.insert(file_name, code, private_script);
                counter += 1;
            }
        } else {
            warn!("couldn't access script folder: {:?}", path.as_ref());
        }

        Ok(counter)
    }

    pub fn count_default_scripts(&self) -> usize {
        self.scripts.iter()
            .filter(|(_, (_, private_script))| !private_script)
            .count()
    }

    pub fn count_private_scripts(&self) -> usize {
        self.scripts.iter()
            .filter(|(_, (_, private_script))| *private_script)
            .count()
    }
}
