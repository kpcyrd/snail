use ::Result;
use std::time::Duration;

use trust_dns_resolver;
use trust_dns_resolver::config::ResolverConfig;
use trust_dns_resolver::config::ResolverOpts;
use trust_dns_resolver::config::NameServerConfig;
use trust_dns_resolver::config::Protocol;
use trust_dns_resolver::lookup_ip::LookupIp;


pub struct Resolver {
    resolver: trust_dns_resolver::Resolver,
}

impl Resolver {
    pub fn with_udp(resolver: &str) -> Result<Resolver> {
        let mut config = ResolverConfig::new();
        config.add_name_server(NameServerConfig {
            socket_addr: resolver.parse()?,
            protocol: Protocol::Udp,
            tls_dns_name: None,
        });

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(1);

        let resolver = trust_dns_resolver::Resolver::new(config, opts)?;

        Ok(Resolver {
            resolver,
        })
    }

    pub fn resolve(&self, name: &str) -> Result<LookupIp> {
        let response = match self.resolver.lookup_ip(name) {
            Ok(response) => response,
            Err(err) => bail!("resolve error: {}", err),
        };
        Ok(response)
    }
}

pub fn resolve(name: &str) -> Result<LookupIp> {
    resolve_with("1.2.3.4:53", name)
}

pub fn resolve_with(resolver: &str, name: &str) -> Result<LookupIp> {
    let resolver = Resolver::with_udp(resolver)?;
    resolver.resolve(name)
}
