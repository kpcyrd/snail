extern crate snail;
extern crate structopt;
extern crate dbus;
extern crate env_logger;
// #[macro_use] extern crate failure;

use structopt::StructOpt;

use snail::Result;
use snail::args::snailctl::{Args, SubCommand};
use snail::decap;


fn run() -> Result<()> {
    let args = Args::from_args();

    let mut env = env_logger::Env::default();
    if args.verbose {
        env = env.filter_or("RUST_LOG", "info");
    }
    env_logger::init_from_env(env);

    match args.subcommand {
        Some(SubCommand::Scan(scan)) => {
            println!("scanning on {:?}", scan.interface);

            // use dbus::{Connection, BusType};

            // let c = Connection::get_private(BusType::System).unwrap();

            unimplemented!()
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
\x1b[33m
    o    o     __ __
     \\  /    '       `
      |/   /     __    \\
    (`  \\ '    '    \\   '
      \\  \\|   |   @_/   |
       \\   \\   \\       /--/
        ` ___ ___ ___ __ '
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
        std::process::exit(1);
    }
}
