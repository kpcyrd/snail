use config::Config;
use dns::{Resolver, DnsResolver};
use errors::{Result, ResultExt};
use scripts::Script;
use web::{self, HttpClient};
use wifi::NetworkStatus;

use std::fs;
use std::path::Path;
use std::sync::Arc;


#[derive(Debug, Clone)]
pub struct Loader {
    scripts: Vec<String>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader {
            scripts: Vec::new(),
        }
    }

    pub fn init<C: HttpClient + 'static, R: DnsResolver + 'static>(&self, http: Arc<C>, resolver: Arc<R>) -> Result<Vec<Script<C, R>>> {
        self.scripts.iter()
            .map(|code| {
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

    pub fn load(&mut self, code: String) -> Result<()> {
        self.scripts.push(code);
        Ok(())
    }

    pub fn load_all_scripts(&mut self, config: &Config) -> Result<usize> {
        let mut counter = 0;
        counter += self.load_default_scripts()?;
        counter += self.load_private_scripts(config)?;
        Ok(counter)
    }

    pub fn load_default_scripts(&mut self) -> Result<usize> {
        self.load_from_folder("/usr/lib/snaild/scripts")
    }

    pub fn load_private_scripts(&mut self, config: &Config) -> Result<usize> {
        let mut counter = 0;

        counter += self.load_from_folder("/etc/snail/scripts/")?;

        for (path, _) in &config.scripts.paths {
            counter += self.load_from_folder(path)?;
        }

        Ok(counter)
    }

    pub fn load_from_folder<P: AsRef<Path>>(&mut self, path: P) -> Result<usize> {
        let mut counter = 0;

        if let Ok(paths) = fs::read_dir(path) {
            for path in paths {
                let path = path?;
                use std::os::unix::ffi::OsStrExt;
                if path.file_name().as_bytes().ends_with(b".lua") {
                    let code = fs::read_to_string(path.path())
                        .context(format!("failed to open {:?}", path.path()))?;
                    self.scripts.push(code);
                    counter += 1;
                }
            }
        }

        // if this fails, ignore the error
        // TODO: maybe print a warning
        Ok(counter)
    }
}
