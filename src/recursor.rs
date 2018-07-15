use errors::{Result, ResultExt};
use config::Config;
use sandbox;

use trust_dns_server::server::{ServerFuture, Request, RequestHandler, ResponseHandler};
use trust_dns_server::authority::{MessageResponseBuilder, AuthLookup};
use trust_dns_proto::op::response_code::ResponseCode;
use futures::{future, Future};
use tokio_udp::UdpSocket;
use tokio::runtime::current_thread::Runtime;

use trust_dns_resolver::Resolver;
use trust_dns_resolver::config::*;
use trust_dns_proto::op::header::Header;
use trust_dns_proto::op::header::MessageType;
use trust_dns_resolver::lookup::Lookup;
use trust_dns_resolver::error::ResolveResult;
use trust_dns_server::authority::authority::LookupRecords;
use trust_dns_proto::rr::{RrsetRecords, RecordType, Record};

use mrsc;

use std::io;
use std::thread;
use std::net::SocketAddr;
use std::time::Instant;
use std::sync::mpsc;


pub struct DnsHandler {
    channel: mrsc::Channel<(String, RecordType), ResolveResult<Lookup>>,
    seccomp_signal: Option<mpsc::Sender<()>>,
}

impl DnsHandler {
    pub fn new(config: &Config) -> Result<DnsHandler> {
        // TODO: dirty hack incoming
        // create a new thread, we can't run the server in the same thread as the resolver
        // the way it's currently built, we can only resolve one record at a time
        let server = mrsc::Server::<(String, RecordType), ResolveResult<Lookup>>::new();
        let channel = server.pop();

        let (tx, rx) = mpsc::channel();
        let danger_disable_seccomp_security = config.security.danger_disable_seccomp_security;

        let resolver_config = match config.dns {
            Some(ref config) => {
                let name_servers = NameServerConfigGroup::from_ips_https(
                    &config.servers,
                    config.port,
                    config.sni.clone(),
                );

                ResolverConfig::from_parts(
                    None, // domain
                    vec![], // search
                    name_servers,
                )
            },
            _ => bail!("dns is not configured"),
        };

        let mut resolver_opts = ResolverOpts::default();
        resolver_opts.use_hosts_file = false;
        let resolver = Resolver::new(resolver_config, resolver_opts)?;

        thread::spawn(move || {
            // block until seccomp can be setup
            rx.recv().unwrap();

            if !danger_disable_seccomp_security {
                sandbox::dns_stage3()
                    .context("sandbox dns_stage3 failed")
                    .unwrap();
            }

            loop {
                let req = server.recv().unwrap();
                debug!("mrsc(server): {:?}", req);
                let (req, (name, query_type)) = req.take();

                // TODO: take the query type from msg
                let response = resolver.lookup(&name, query_type);

                debug!("mrsc: {:?}", response);
                req.reply(response).unwrap();
            }
        });

        Ok(DnsHandler {
            channel,
            seccomp_signal: Some(tx),
        })
    }

    pub fn run(mut self, addr: &SocketAddr, config: &Config) -> Result<()> {
        let mut io_loop = Runtime::new()?;
        let seccomp_signal = self.seccomp_signal.take().unwrap();
        let server = ServerFuture::new(self);

        let socket = UdpSocket::bind(addr)?;

        let server_future: Box<Future<Item=(), Error=()> + Send> = Box::new(future::lazy(move || {
            server.register_socket(socket);
            info!("dns recursor starting up");
            future::empty()
        }));

        // signal that seccomp can be activated
        seccomp_signal.send(()).unwrap();

        if !config.security.danger_disable_seccomp_security {
            sandbox::dns_stage3()
                .context("sandbox dns_stage3 failed")?;
        }

        if let Err(e) = io_loop.block_on(server_future.map_err(|_| io::Error::new(
            io::ErrorKind::Interrupted,
            "Server stopping due to interruption",
        ))) {
            bail!("failed to listen: {}", e);
        }

        Ok(())
    }
}

impl RequestHandler for DnsHandler {
    fn handle_request<R: ResponseHandler>(&self, request: &Request, response_handle: R) -> io::Result<()> {
        debug!("dns(req): {}: {:?}", request.src, request.message);

        let queries = request.message.queries();
        let query = queries.get(0).expect("failed to get the first query from dns request");

        let lookup = (query.name().to_string(), query.query_type());
        let req = self.channel.req(lookup).unwrap();
        let resp = req.recv().unwrap();
        debug!("dns(recursor): {:?}", resp);

        let mut answers = Vec::new();
        let mut msg = MessageResponseBuilder::new(Some(request.message.raw_queries()));

        let msg = match resp {
            Ok(resp) => {
                // msg.set_id(request.message.id());
                let mut header = Header::new();
                header.set_id(request.message.id());
                header.set_message_type(MessageType::Response);
                header.set_recursion_desired(true);
                header.set_recursion_available(true);

                let now = Instant::now();
                let ttl = resp.valid_until();
                let ttl = if now < ttl {
                    ttl.duration_since(now).as_secs() as u32
                } else {
                    0
                };

                answers.extend(resp.iter()
                    .map(|r| {
                        let name = query.name().to_owned();
                        Record::from_rdata(name.into(), ttl, query.query_type(), r.to_owned())
                    }));

                msg.answers(AuthLookup::Records(LookupRecords::RecordsIter(RrsetRecords::RecordsOnly(answers.iter()))));
                msg.build(header)
            },
            Err(_) => {
                msg.error_msg(request.message.id(), request.message.op_code(), ResponseCode::ServFail)
            },
        };

        debug!("dns(resp): {}: {:?}", request.src, msg);
        response_handle.send(msg)?;

        Ok(())
    }
}
