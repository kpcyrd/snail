use dns::{Resolver, DnsResolver};
use scripts::Script;
use web::{self, HttpClient};
use wifi::NetworkStatus;
use errors::Result;

use std::fs;
use std::env;
use std::path::Path;
use std::sync::Arc;


#[derive(Debug, Clone)]
pub struct Loader<C: HttpClient + 'static, R: DnsResolver + 'static> {
    http: Arc<C>,
    resolver: Arc<R>,
}

impl Loader<web::Client<Resolver>, Resolver> {
    pub fn from_status(status: &Option<NetworkStatus>) -> Result<Loader<web::Client<Resolver>, Resolver>> {
        match status {
            Some(status) => {
                let resolver = Resolver::with_udp(&status.dns)?;
                let http = Arc::new(web::Client::new(resolver));
                let resolver = Arc::new(Resolver::with_udp(&status.dns)?);

                Ok(Loader::new(http, resolver))
            },
            None => Ok(Loader::default()),
        }
    }
}

impl<C: HttpClient, R: DnsResolver> Loader<C, R> {
    #[inline]
    pub fn new(http: Arc<C>, resolver: Arc<R>) -> Loader<C, R> {
        Loader {
            http,
            resolver,
        }
    }

    pub fn load(&self, code: String) -> Result<Script<C, R>> {
        Script::load(code, self.http.clone(), self.resolver.clone())
    }

    pub fn load_all_scripts(&self) -> Result<Vec<Script<C, R>>> {
        let mut scripts = Vec::new();
        scripts.extend(self.load_default_scripts()?);
        scripts.extend(self.load_private_scripts()?);
        Ok(scripts)
    }

    pub fn load_default_scripts(&self) -> Result<Vec<Script<C, R>>> {
        self.load_from_folder("/usr/lib/snaild/scripts")
    }

    pub fn load_private_scripts(&self) -> Result<Vec<Script<C, R>>> {
        match env::home_dir() {
            Some(home) => self.load_from_folder(home.join(".config/snaild/scripts/")),
            None => Ok(Vec::new()),
        }
    }

    pub fn load_from_folder<P: AsRef<Path>>(&self, path: P) -> Result<Vec<Script<C, R>>> {
        match fs::read_dir(path) {
            Ok(paths) => paths
                            .map(|x| Script::load_from_path(x?.path(), self.http.clone(), self.resolver.clone()))
                            .collect(),
            // if this fails, ignore the error
            Err(_) => Ok(Vec::new()),
        }
    }
}

impl Default for Loader<web::Client<Resolver>, Resolver> {
    fn default() -> Loader<web::Client<Resolver>, Resolver> {
        let resolver = Arc::new(Resolver::cloudflare());
        let http = Arc::new(web::Client::new(Resolver::cloudflare()));
        Loader::new(http, resolver)
    }
}
