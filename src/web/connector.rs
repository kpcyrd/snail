use hyper_rustls::HttpsConnector;
use hyper::rt::Future;
use hyper::client::connect::{self, Connect};
use hyper::client::connect::HttpConnector;
use hyper::client::connect::Destination;
use futures::{Poll};
use rustls::ClientConfig;
use webpki_roots;
use ct_logs;

use std::io;
use std::net::IpAddr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use errors::Result;


pub struct Connector<T> {
    http: T,
    // resolver: ResolverFuture,
    records: Arc<Mutex<HashMap<String, IpAddr>>>,
}

impl<T> Connector<T> {
    pub fn resolve_dest(&self, mut dest: Destination) -> Result<Destination> {
        let ip = {
            let cache = self.records.lock().unwrap();
            cache.get(dest.host()).map(|x| x.to_owned())
        };

        let ip = match ip {
            Some(ip) => ip,
            None => bail!("host wasn't pre-resolved"),
        };

        dest.set_host(&ip.to_string())?;

        Ok(dest)
    }
}

impl Connector<HttpConnector> {
    pub fn new(records: Arc<Mutex<HashMap<String, IpAddr>>>) -> Connector<HttpConnector> {
        let mut http = HttpConnector::new(4);
        http.enforce_http(false);
        Connector {
            http,
            records,
        }
    }

    pub fn https(records: Arc<Mutex<HashMap<String, IpAddr>>>) -> HttpsConnector<Connector<HttpConnector>> {
        let http = Connector::new(records);

        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        config.ct_logs = Some(&ct_logs::LOGS);

        HttpsConnector::from((http, config))
    }
}

impl<T> Connect for Connector<T>
where
    T: Connect<Error=io::Error>,
    T: Clone,
    T: 'static,
    T::Transport: 'static,
    T::Future: 'static,
{
    type Transport = T::Transport;
    type Error = io::Error;
    type Future = Connecting<T::Transport>;

    fn connect(&self, dest: connect::Destination) -> Self::Future {
        debug!("original destination: {:?}", dest);
        let dest = self.resolve_dest(dest).expect("resolve failed"); // TODO
        debug!("resolved destination: {:?}", dest);
        let connecting = self.http.connect(dest);
        let fut = Box::new(connecting);
        Connecting(fut)

        /*
        // async implementation
        // compiles but hangs forever
        println!("creating resolve");
        let resolving = self.resolve_dest(&dest);

        let http = self.http.clone();
        println!("chaining resolve");
        let fut = Box::new(resolving.and_then(move |records| {
            // unimplemented!()
            println!("records: {:?}", records);
            http.connect(dest)
        }));
        println!("returning future");
        Connecting(fut)
        */
    }
}

/// A Future representing work to connect to a URL
pub struct Connecting<T>(
    Box<Future<Item = (T, connect::Connected), Error = io::Error> + Send>,
);

impl<T> Future for Connecting<T> {
    type Item = (T, connect::Connected);
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.0.poll()
    }
}
