use hyper::{self, Body};
use http::response::Parts;
use hyper_rustls::HttpsConnector;
use hyper::rt::Future;
use hyper::client::connect::HttpConnector;
use http::Request;

use tokio_core::reactor;
use futures::{future, Stream};

use std::net::IpAddr;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use http::Uri;
use errors::Result;

mod connector;
use self::connector::Connector;
use dns::DnsResolver;
pub mod structs;


#[derive(Debug)]
pub struct Client<R: DnsResolver> {
    client: hyper::Client<HttpsConnector<Connector<HttpConnector>>>,
    resolver: R,
    records: Arc<Mutex<HashMap<String, IpAddr>>>,
}

impl<R: DnsResolver> Client<R> {
    pub fn new(resolver: R) -> Client<R> {
        let records = Arc::new(Mutex::new(HashMap::new()));
        let https = Connector::https(records.clone());
        let client = hyper::Client::builder()
            .keep_alive(false)
            .build::<_, hyper::Body>(https);

        Client {
            client,
            resolver,
            records,
        }
    }

    pub fn pre_resolve(&self, uri: &Uri) -> Result<()> {
        let host = match uri.host() {
            Some(host) => host,
            None => bail!("url has no host"),
        };

        let record = self.resolver.resolve(&host)?;
        match record.into_iter().next() {
            Some(record) => {
                let mut cache = self.records.lock().unwrap();
                cache.insert(host.to_string(), record);
            },
            None => bail!("no record found"),
        }
        Ok(())
    }

    pub fn get(&self, url: &str) -> Result<Response> {
        info!("sending request to {:?}", url);
        let url = url.parse::<Uri>()?;

        self.pre_resolve(&url)?;

        let mut request = Request::builder();
        let request = request.uri(url.clone())
               .body(Body::empty())?;

        self.request(&url, request)
    }
}

pub trait HttpClient {
    fn request(&self, url: &Uri, request: Request<hyper::Body>) -> Result<Response>;
}

impl<R: DnsResolver> HttpClient for Client<R> {
    fn request(&self, url: &Uri, request: Request<hyper::Body>) -> Result<Response> {
        info!("sending request to {:?}", url);

        self.pre_resolve(url)?;

        let mut core = reactor::Core::new()?;
        let (parts, body) = core.run(self.client.request(request).and_then(|res| {
            debug!("http response: {:?}", res);
            let (parts, body) = res.into_parts();
            let body = body.concat2();
            (future::ok(parts), body)
        }))?;

        let body = String::from_utf8_lossy(&body);
        let reply = Response::from((parts, body.to_string()));
        info!("got reply {:?}", reply);
        Ok(reply)
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub cookies: Vec<String>,
    pub body: String,
}

impl From<(Parts, String)> for Response {
    fn from(x: (Parts, String)) -> Response {
        let parts = x.0;
        let body = x.1;

        let cookies = parts.headers.get_all("set-cookie").into_iter()
                        .flat_map(|x| x.to_str().map(|x| x.to_owned()).ok())
                        .collect();

        let mut headers = HashMap::new();

        for (k, v) in parts.headers {
            if let Some(k) = k {
                if let Ok(v) = v.to_str() {
                    let k = String::from(k.as_str());
                    let v = String::from(v);

                    headers.insert(k, v);
                }
            }
        }

        Response {
            status: parts.status.as_u16(),
            headers,
            cookies,
            body,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use dns::Resolver;

    #[test]
    #[ignore]
    fn verify_200_http() {
        let resolver = Resolver::cloudflare();

        let client = Client::new(resolver);
        let reply = client.get("http://httpbin.org/anything").expect("request failed");
        assert_eq!(reply.status, 200);
    }

    #[test]
    #[ignore]
    fn verify_200_https() {
        let resolver = Resolver::cloudflare();

        let client = Client::new(resolver);
        let reply = client.get("https://httpbin.org/anything").expect("request failed");
        assert_eq!(reply.status, 200);
    }

    #[test]
    #[ignore]
    fn verify_302() {
        let resolver = Resolver::cloudflare();

        let client = Client::new(resolver);
        let reply = client.get("https://httpbin.org/redirect-to?url=/anything&status=302").expect("request failed");
        assert_eq!(reply.status, 302);
    }
}
