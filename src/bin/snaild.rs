extern crate snail;
extern crate structopt;
extern crate env_logger;
// extern crate colored;
#[macro_use] extern crate log;
// #[macro_use] extern crate failure;

use structopt::StructOpt;
// use colored::Colorize;

use snail::Result;
use snail::args::snaild::Args;
use snail::dhcp;
use snail::ipc::{Server, Client, CtlRequest, CtlReply};

use std::env;
use std::thread;
// use std::sync::mpsc;

const SOCK: &str = "ipc:///tmp/snail.sock";


fn dhcp_thread(interface: &str, hook: &str) -> Result<()> {
    info!("starting dhcpcd");
    dhcp::run_dhcpcd(interface, hook)?;
    info!("dhcpcd exited");
    Ok(())
}

fn zmq_thread() -> Result<()> {
    let mut server = Server::bind(SOCK)?;

    loop {
        let msg = server.recv()?;

        let reply = match msg {
            CtlRequest::Ping => CtlReply::Pong,
            CtlRequest::DhcpEvent(event) => {
                info!("dhcp: {:?}", event);

                use dhcp::UpdateMessage;
                match event.message {
                    Some(UpdateMessage::Carrier) => {
                        info!("got carrier");
                    },
                    Some(UpdateMessage::Bound(_net)) => {
                        info!("TODO: bound");
                    },
                    Some(UpdateMessage::Reboot(_net)) => {
                        info!("TODO: bound");
                    },
                    Some(UpdateMessage::Renew(_net)) => {
                        // ignore
                    },
                    Some(UpdateMessage::NoCarrier) => {
                        info!("carrier lost");
                    },
                    None => (),
                };

                CtlReply::Ack
            },
        };

        server.reply(&reply)?;
    }
}

fn run_daemon(args: Args) -> Result<()> {
    let hook = {
        let h = env::current_exe().unwrap();
        h.to_str().unwrap().to_string()
    };
    let interface = args.interface;

    let t1 = thread::spawn(move || {
        dhcp_thread(&interface, &hook).expect("dhcp_thread");
    });

    let t2 = thread::spawn(move || {
        zmq_thread().expect("zmq_thread");
    });

    t1.join().expect("dhcp thread failed");
    t2.join().expect("zmq thread failed");

    Ok(())
}

fn notify_daemon() -> Result<()> {
    let event = dhcp::read_dhcp_env()?;

    let mut client = Client::connect(SOCK)?;
    client.send(&CtlRequest::DhcpEvent(event))?;

    /*
    println!("{}", r"//////////////////////////////".red());
    println!("{:#?}", foo);
    println!("{}", r"\\\\\\\\\\\\\\\\\\\\\\\\\\\\\\".red());
    */

    Ok(())
}


fn run() -> Result<()> {
    match env::var("reason") {
        // we've been invoked by dhcpcd
        // send the event to snaild
        Ok(_)  => {
            let env = env_logger::Env::default()
                .filter_or("RUST_LOG", "info");
            env_logger::init_from_env(env);
            notify_daemon()
        },
        // else, start daemon
        Err(_) => {
            let args = Args::from_args();

            let mut env = env_logger::Env::default();
            if args.verbose {
                env = env.filter_or("RUST_LOG", "debug");
            } else {
                env = env.filter_or("RUST_LOG", "info");
            }
            env_logger::init_from_env(env);

            run_daemon(args)
        }
    }
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
