use ::Result;
use std::time::Duration;
use std::net::IpAddr;
use std::fmt;

use futures::Future;
use futures::Poll;
use tokio_core::reactor;
use trust_dns_resolver as tdr;
use trust_dns_resolver::lookup_ip::LookupIp;
use trust_dns_resolver::config::{ResolverConfig,
                                 ResolverOpts,
                                 NameServerConfig,
                                 Protocol};

use std::io;
use std::net::SocketAddr;


pub struct Resolver {
    resolver: tdr::Resolver,
}

impl Resolver {
    pub fn cloudflare() -> Resolver {
        Resolver::with_udp_addr(&["1.1.1.1:53".parse().unwrap(),
                                  "1.0.0.1:53".parse().unwrap()]).unwrap()
    }

    pub fn with_udp_addr(recursors: &[SocketAddr]) -> Result<Resolver> {
        let mut config = ResolverConfig::new();

        for recursor in recursors {
            config.add_name_server(NameServerConfig {
                socket_addr: recursor.to_owned(),
                protocol: Protocol::Udp,
                tls_dns_name: None,
            });
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(1);

        let resolver = tdr::Resolver::new(config, opts)?;

        Ok(Resolver {
            resolver,
        })
    }

    pub fn with_udp(recursors: &[IpAddr]) -> Result<Resolver> {
        let recursors = recursors.into_iter()
                            .map(|x| SocketAddr::new(x.to_owned(), 53))
                            .collect::<Vec<_>>();
        Resolver::with_udp_addr(&recursors)
    }

    #[inline]
    pub fn transform(lookup: LookupIp) -> Vec<IpAddr> {
        lookup.iter().collect()
    }
}

impl fmt::Debug for Resolver {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Resolver {{ ... }}")
    }
}

pub trait DnsResolver {
    fn resolve(&self, name: &str) -> Result<Vec<IpAddr>>;
}

impl DnsResolver for Resolver {
    fn resolve(&self, name: &str) -> Result<Vec<IpAddr>> {
        let response = match self.resolver.lookup_ip(name) {
            Ok(response) => Resolver::transform(response),
            Err(err) => bail!("resolve error: {}", err),
        };
        Ok(response)
    }
}

pub struct ResolverFuture {
    resolver: tdr::ResolverFuture,
}

impl ResolverFuture {
    pub fn cloudflare() -> ResolverFuture {
        ResolverFuture::with_udp_addr(&[String::from("1.1.1.1:53"),
                                        String::from("1.0.0.1:53")]).unwrap()
    }

    pub fn with_udp_addr(recursors: &[String]) -> Result<ResolverFuture> {
        let mut config = ResolverConfig::new();

        for recursor in recursors {
            config.add_name_server(NameServerConfig {
                socket_addr: recursor.parse()?,
                protocol: Protocol::Udp,
                tls_dns_name: None,
            });
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(1);

        let mut core = reactor::Core::new()?;
        let resolver = core.run(tdr::ResolverFuture::new(config, opts));

        let resolver = match resolver {
            Ok(resolver) => resolver,
            Err(_) => bail!("resolver init error"), // TODO
        };

        Ok(ResolverFuture {
            resolver,
        })
    }

    pub fn with_udp(recursors: &[String]) -> Result<ResolverFuture> {
        let recursors = recursors.iter()
                            .map(|x| format!("{}:53", x))
                            .collect::<Vec<_>>();
        ResolverFuture::with_udp_addr(&recursors)
    }

    pub fn resolve(&self, name: &str) -> Resolving {
        let fut = self.resolver.lookup_ip(name)
            .map(|lookup| {
                Resolver::transform(lookup)
            })
            .map_err(|err| {
                io::Error::new(io::ErrorKind::Other, format!("{:?}", err)) // TODO
            });
        Resolving(Box::new(fut))
    }
}

/// A Future representing work to connect to a URL
pub struct Resolving(
    Box<Future<Item = Vec<IpAddr>, Error = io::Error> + Send>,
);

impl Future for Resolving {
    type Item = Vec<IpAddr>;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}
