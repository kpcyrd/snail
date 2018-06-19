extern crate snail;
extern crate structopt;
extern crate dbus;
extern crate env_logger;
extern crate colored;
extern crate reduce;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;

use structopt::StructOpt;
use colored::Colorize;
use reduce::Reduce;
use failure::ResultExt;

use snail::Result;
use snail::args;
use snail::args::snailctl::{Args, SubCommand};
use snail::decap;
use snail::dns::{Resolver, DnsResolver};
// use snail::dhcp;
use snail::utils;
use snail::ipc::Client;


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

    match args.subcommand {
        Some(SubCommand::Scan(scan)) => {
            // println!("scanning on {:?}", scan.interface);

            // there is no network status, so we just use a default environment
            let loader = snail::scripts::Loader::default();
            let scripts = loader.load_all_scripts()?;

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
            // snail::scripts::loader::loader

            let mut client = Client::connect(&args.socket)?;
            let status = match client.status()? {
                Some(status) => status,
                None => bail!("not connected to a network"),
            };

            let resolver = Resolver::with_udp(&status.dns)?;

            let walled_garden = match decap::detect_walled_garden(resolver)? {
                Some(walled_garden) => walled_garden,
                None => {
                    println!("[+] no walled garden detected");
                    return Ok(());
                },
            };

            println!("[!] walled garden connection detected!");
            println!("{:?}", walled_garden);
        },
        Some(SubCommand::Status(_decap)) => {
            let mut client = Client::connect(&args.socket)?;

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
            let mut client = Client::connect(&args.socket)?;

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
        Some(SubCommand::Http(_http)) => {
            println!("unimplemented");
        },
        Some(SubCommand::BashCompletion) => {
            args::gen_completions::<args::snailctl::Args>("snailctl");
        },
        None => {
            // use empty network status, we don't support function calls here
            let loader = snail::scripts::Loader::default();

            let default_scripts = loader.load_default_scripts()?;
            let private_scripts = loader.load_private_scripts()?;
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
", default_scripts.len(), private_scripts.len());
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
