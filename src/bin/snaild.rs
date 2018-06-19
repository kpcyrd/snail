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
use snail::decap;
use snail::dhcp;
use snail::ipc::{Server, Client, CtlRequest, CtlReply};
use snail::wifi::NetworkStatus;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;


fn dhcp_thread(interface: &str, hook: &str) -> Result<()> {
    let mut conf = env::temp_dir();
    conf.push("snaild-dhcpcd.conf"); // TODO: this is not secure

    let mut f = File::create(&conf)?;
    f.write(br#"
# A sample configuration for dhcpcd.
# See dhcpcd.conf(5) for details.

# Allow users of this group to interact with dhcpcd via the control socket.
#controlgroup wheel

# Inform the DHCP server of our hostname for DDNS.
hostname

# Use the hardware address of the interface for the Client ID.
#clientid
# or
# Use the same DUID + IAID as set in DHCPv6 for DHCPv4 ClientID as per RFC4361.
# Some non-RFC compliant DHCP servers do not reply with this set.
# In this case, comment out duid and enable clientid above.
duid

# Persist interface configuration when dhcpcd exits.
#persistent

# Rapid commit support.
# Safe to enable by default because it requires the equivalent option set
# on the server to actually work.
option rapid_commit

# A list of options to request from the DHCP server.
option domain_name_servers, domain_name, domain_search, host_name
option classless_static_routes
# Respect the network MTU. This is applied to DHCP routes.
option interface_mtu

# Most distributions have NTP support.
#option ntp_servers

# A ServerID is required by RFC2131.
require dhcp_server_identifier

# Generate SLAAC address using the Hardware Address of the interface
#slaac hwaddr
# OR generate Stable Private IPv6 Addresses based from the DUID
slaac private
noipv4ll
"#)?;
    f.flush()?;

    info!("starting dhcpcd");
    dhcp::run_dhcpcd(&conf.to_str().unwrap(), interface, hook)?;
    info!("dhcpcd exited");
    Ok(())
}

fn decap_thread_loop(status: Arc<Mutex<Option<NetworkStatus>>>, msg: NetworkStatus) -> Result<()> {
    debug!("rx: {:?}", msg);
    thread::sleep(Duration::from_secs(1));

    let mut status = status.lock().unwrap();
    if let Some(ref mut status) = status.deref_mut() {
        decap::decap(status, &msg.dns)?;
    } else {
        warn!("not connected to a network");
    }

    Ok(())
}

fn decap_thread(status: Arc<Mutex<Option<NetworkStatus>>>, rx: mpsc::Receiver<NetworkStatus>) -> Result<()> {
    for msg in rx {
        if let Err(error) = decap_thread_loop(status.clone(), msg) {
            error!("error in decap thread: {:?}", error);
        }
    }

    Ok(())
}

fn zmq_thread(socket: &str, status: Arc<Mutex<Option<NetworkStatus>>>, tx: mpsc::Sender<NetworkStatus>) -> Result<()> {
    let mut server = Server::bind(socket)?;

    loop {
        let msg = server.recv()?;

        let reply = match msg {
            CtlRequest::Ping => CtlReply::Pong,
            CtlRequest::DhcpEvent(event) => {
                debug!("dhcp: {:?}", event);

                use dhcp::UpdateMessage;
                match event.message {
                    Some(UpdateMessage::Carrier) => {
                        info!("got carrier");
                    },
                    Some(UpdateMessage::Bound(net)) => {
                        info!("successful dhcp bound");
                        let mut status = status.lock().unwrap();
                        let network = NetworkStatus::new(event.ssid, net);
                        *status = Some(network.clone());
                        tx.send(network)?;
                    },
                    Some(UpdateMessage::Reboot(net)) => {
                        info!("successful dhcp reboot");
                        let mut status = status.lock().unwrap();
                        let network = NetworkStatus::new(event.ssid, net);
                        *status = Some(network.clone());
                        tx.send(network)?;
                    },
                    Some(UpdateMessage::Renew(_net)) => {
                        // ignore
                        debug!("dhcp renewed");
                    },
                    Some(UpdateMessage::NoCarrier) => {
                        info!("carrier lost");
                        let mut status = status.lock().unwrap();
                        *status = None;
                    },
                    Some(UpdateMessage::Stopped) => {
                        // ignore
                    },
                    None => (),
                };
                // TODO: STOPPED should shutdown everything

                CtlReply::Ack
            },
            CtlRequest::StatusRequest => {
                let mut status = status.lock().unwrap();
                CtlReply::Status(status.clone())
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

    let socket = args.socket;
    let interface = args.interface;
    let status = Arc::new(Mutex::new(None));
    let (tx, rx) = mpsc::channel();

    let t1 = {
        thread::spawn(move || {
            dhcp_thread(&interface, &hook).expect("dhcp_thread");
        })
    };

    let t2 = {
        let status = status.clone();
        thread::spawn(move || {
            decap_thread(status, rx).expect("decap_thread");
        })
    };

    let t3 = {
        let status = status.clone();
        thread::spawn(move || {
            zmq_thread(&socket, status, tx).expect("zmq_thread");
        })
    };

    t1.join().expect("dhcp thread failed");
    t2.join().expect("decap thread failed");
    t3.join().expect("zmq thread failed");

    Ok(())
}

fn notify_daemon(socket: &str) -> Result<()> {
    let event = dhcp::read_dhcp_env()?;

    // println!("{:?}", event);

    // this can prevent dhcpcd from shutting down if the zmq thread already stopped
    if event.message == Some(dhcp::UpdateMessage::Stopped) {
        return Ok(());
    }

    debug!("event: {:?}", event);
    let mut client = Client::connect(socket)?;
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
            notify_daemon("ipc:///tmp/snail.sock") // TODO: hardcoded path
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
