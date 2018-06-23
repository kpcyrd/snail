use dns::{Resolver, DnsResolver};
use scripts::Script;
use web::{self, HttpClient};
use wifi::NetworkStatus;
use errors::{Result, ResultExt};

use std::fs;
use std::env;
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

    pub fn init_all_scripts_default() -> Result<Vec<Script<web::Client<Resolver>, Resolver>>> {
        let mut loader = Loader::new();
        loader.load_all_scripts()?;

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

    pub fn load_all_scripts(&mut self) -> Result<()> {
        self.load_default_scripts()?;
        self.load_private_scripts()?;
        Ok(())
    }

    pub fn load_default_scripts(&mut self) -> Result<()> {
        self.load_from_folder("/usr/lib/snaild/scripts")
    }

    pub fn load_private_scripts(&mut self) -> Result<()> {
        if let Some(home) = env::home_dir() {
            self.load_from_folder(home.join(".config/snaild/scripts/"))?;
        }
        Ok(())
    }

    pub fn load_from_folder<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if let Ok(paths) = fs::read_dir(path) {
            for path in paths {
                let path = path?;
                let code = fs::read_to_string(path.path())
                    .context(format!("failed to open {:?}", path.path()))?;
                self.scripts.push(code);
            }
        }

        // if this fails, ignore the error
        // TODO: maybe print a warning
        Ok(())
    }
}
