extern crate snail;
extern crate structopt;
extern crate dbus;
extern crate env_logger;
extern crate colored;
#[macro_use] extern crate log;
// #[macro_use] extern crate failure;

use structopt::StructOpt;
use colored::Colorize;

use snail::Result;
use snail::args::snailctl::{Args, SubCommand};
use snail::decap;
use snail::dns;
use snail::utils;


fn run() -> Result<()> {
    let args = Args::from_args();

    let mut env = env_logger::Env::default();
    if args.verbose {
        env = env.filter_or("RUST_LOG", "info");
    }
    env_logger::init_from_env(env);

    match args.subcommand {
        Some(SubCommand::Scan(scan)) => {
            // println!("scanning on {:?}", scan.interface);

            let scripts = snail::scripts::load_all_scripts()?;

            let networks = utils::scan_wifi(&scan.interface)?;
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

            let walled_garden = match decap::detect_walled_garden()? {
                Some(walled_garden) => walled_garden,
                None => {
                    println!("[+] no walled garden detected");
                    return Ok(());
                },
            };

            println!("[!] walled garden connection detected!");
            println!("{:?}", walled_garden);
        },
        None => {
            let default_scripts = snail::scripts::load_default_scripts()?;
            let private_scripts = snail::scripts::load_private_scripts()?;
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
