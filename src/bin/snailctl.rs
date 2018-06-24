#![warn(unused_extern_crates)]
extern crate snail;
extern crate structopt;
// extern crate dbus;
extern crate env_logger;
extern crate colored;
extern crate reduce;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate http;
extern crate hyper;

use structopt::StructOpt;
use colored::Colorize;
use reduce::Reduce;

use http::{Request, Uri};
use hyper::Body;

use snail::args;
use snail::args::snailctl::{Args, SubCommand};
use snail::config;
use snail::decap;
use snail::dns::{Resolver, DnsResolver};
use snail::errors::{Result, ResultExt};
use snail::ipc::Client;
use snail::sandbox;
use snail::scripts::Loader;
use snail::utils;
use snail::web::{self, HttpClient};


fn run() -> Result<()> {
    let args = Args::from_args();

    let mut env = env_logger::Env::default();
    match args.verbose {
        0 => (),
        1 => {
            env = env.filter_or("RUST_LOG", "info");
        },
        _ => {
            env = env.filter_or("RUST_LOG", "debug");
        },
    };
    env_logger::init_from_env(env);

    let config = config::read_from(config::PATH)
                    .context("failed to load config")?;
    debug!("config: {:?}", config);
    let socket = args.socket.unwrap_or(config.daemon.socket.clone());

    match args.subcommand {
        Some(SubCommand::Scan(scan)) => {
            // println!("scanning on {:?}", scan.interface);

            // there is no network status, so we just use a default environment
            let scripts = Loader::init_all_scripts_default(&config)?;

            let networks = utils::scan_wifi(&scan.interface)
                            .context("scan_wifi failed")?;
            for network in networks {
                let encryption = match network.encryption.as_str() {
                    "on"  => "on ".red().to_string(),
                    "off" => "off".green().to_string(),
                    _     => String::new(),
                };

                let mut has_script = false;
                for script in &scripts {
                    if script.detect_network(&network.essid)? {
                        info!("found script! {:?}", script);
                        has_script = true;
                        break;
                    }
                }

                let script_indicator = if has_script {
                    "$".green().to_string()
                } else {
                    String::from(" ")
                };

                println!(" {} {:?} {:28} encryption={} signal={:?} dBm channel={:?}",
                         script_indicator,
                         network.ap,
                         format!("{:?}", network.essid),
                         encryption,
                         network.signal,
                         network.channel);
            }
        },
        Some(SubCommand::Decap(_decap)) => {
            if !config.danger_disable_seccomp_security {
                sandbox::decap_stage1()?;
            }

            let mut loader = Loader::new();
            loader.load_all_scripts(&config)?;

            let mut client = Client::connect(&socket)?;
            let mut status = match client.status()? {
                Some(status) => status,
                None => bail!("not connected to a network"),
            };

            let dns = status.dns.clone();
            if !config.danger_disable_seccomp_security {
                // TODO: we can't call sandbox::decap_stage2 because we might not be able to chroot
                sandbox::seccomp::decap_stage2()?;
            }
            // TODO: there's no output here unless -v is provided
            decap::decap(&loader, &mut status, &dns)?;
        },
        Some(SubCommand::Status(_status)) => {
            let mut client = Client::connect(&socket)?;

            match client.status()? {
                Some(status) => {
                    println!("network: {}", match status.ssid {
                        Some(ssid) => format!("{:?}", ssid).green(),
                        None       => "unknown".yellow(),
                    });
                    println!("router:  {:?}", status.router);
                    println!("dns:     [{}]", status.dns.iter()
                                                .map(|x| x.to_string())
                                                .reduce(|a, b| a + ", " + &b)
                                                .unwrap_or_else(|| String::new()));
                    println!("uplink:  {}", match status.has_uplink {
                        Some(true)  => "yes".green(),
                        Some(false) => "no".red(),
                        None        => "unknown".yellow(),
                    });
                    println!("script:  {}", match status.script_used {
                        Some(script) => format!("{:?}", script),
                        None         => "none".to_string(),
                    });
                },
                None => {
                    println!("network: {}", "none".red());
                }
            }
        },
        Some(SubCommand::Dns(dns)) => {
            let mut client = Client::connect(&socket)?;

            match client.status()? {
                Some(status) => {
                    let resolver = Resolver::with_udp(&status.dns)?;
                    for ip in resolver.resolve(&dns.query)? {
                        println!("{}", ip);
                    }
                },
                None => bail!("no active network"),
            }
        },
        Some(SubCommand::Http(http)) => {
            let mut client = Client::connect(&socket)?;

            let status = match client.status()? {
                Some(status) => status,
                None => bail!("no active network"),
            };

            let resolver = Resolver::with_udp(&status.dns)?;
            let client = web::Client::new(resolver);

            let url = http.url.parse::<Uri>()?;

            let mut request = Request::builder();
            let request = request.uri(url.clone())
                   .method(http.method.as_str())
                   .body(Body::empty())?;

            let res = client.request(&url, request)?;
            debug!("{:?}", res);

            info!("status: {}", res.status);
            for (key, value) in &res.headers {
                info!("{:?}: {:?}", key, value);
            }

            print!("{}", res.body);
        },
        Some(SubCommand::BashCompletion) => {
            args::gen_completions::<args::snailctl::Args>("snailctl");
        },
        None => {
            // use empty network status, we don't support function calls here
            let mut loader = Loader::new();
            let default_scripts = loader.load_default_scripts()?;
            let private_scripts = loader.load_private_scripts(&config)?;

            print!("snailctl - parasitic network manager

\x1b[32m    o    o     \x1b[33m__ __
\x1b[32m     \\  /    \x1b[33m'       `
\x1b[32m      |/   \x1b[33m/     __    \\
\x1b[32m    (`  \\ \x1b[33m'    '    \\   '
\x1b[32m      \\  \\\x1b[33m|   |   @_/   |
\x1b[32m       \\   \x1b[33m\\   \\       /\x1b[32m--/
\x1b[32m        ` ___ ___ ___ __ '
\x1b[0m
-=[ default scripts: {}
-=[ private scripts: {}
", default_scripts, private_scripts);
        },
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.causes().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
