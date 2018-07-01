#![warn(unused_extern_crates)]
extern crate snail;
extern crate structopt;
extern crate env_logger;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate tempfile;
extern crate serde_json;

use structopt::StructOpt;

use snail::args::snaild::{Args, SubCommand};
use snail::config::{self, Config};
use snail::decap;
use snail::dhcp;
use snail::errors::{Result, ResultExt};
use snail::ipc::{Server, Client, CtlRequest, CtlReply};
use snail::sandbox;
use snail::scripts::Loader;
use snail::wifi::NetworkStatus;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use std::process::{Command, Child, Stdio};
use std::io::{self, BufReader};


fn dhcp_thread(interface: &str, hook: &str) -> Result<()> {
    let dir = tempfile::tempdir()?;
    let conf = dir.path().join("snaild-dhcpcd.conf");

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

fn decap_thread_loop(loader: &Loader, status: &mut Option<NetworkStatus>, msg: NetworkStatus) -> Result<()> {
    debug!("rx: {:?}", msg);
    thread::sleep(Duration::from_secs(1));

    if let Some(ref mut status) = status {
        // TODO: there should be a way to force decap for some networks
        decap::decap(loader, status, &msg.dns, false)?;
    } else {
        warn!("not connected to a network");
    }

    Ok(())
}

fn decap_thread(_socket: &str, config: &Config) -> Result<()> {
    if !config.danger_disable_seccomp_security {
        sandbox::decap_stage1()
            .context("sandbox decap_stage1 failed")?;
    }

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    let mut loader = Loader::new();
    loader.load_all_scripts(config)?;

    if !config.danger_disable_seccomp_security {
        sandbox::decap_stage2()
            .context("sandbox decap_stage2 failed")?;
    }

    // let mut client = Client::connect(socket)?;
    // TODO: only works with chroot
    let mut client = Client::connect("ipc:///snail.sock")?;
    // ensure the connection is fully setup
    client.ping()?;

    if !config.danger_disable_seccomp_security {
        sandbox::decap_stage3()
            .context("sandbox decap_stage2 failed")?;
    }

    for msg in reader.lines() {
        debug!("got event for decap: {:?}", msg);
        let msg = msg?;
        let msg = serde_json::from_str(&msg)?;

        let mut status = client.status()?;
        debug!("got current network status");

        if let Err(error) = decap_thread_loop(&loader, &mut status, msg) {
            error!("error in decap thread: {:?}", error);
        } else {
            client.set_status(status)?;
            debug!("sent network status update");
        }
    }

    Ok(())
}

fn send_to_child(child: &mut Child, status: NetworkStatus) -> Result<()> {
    if let Some(stdin) = &mut child.stdin {
        debug!("sending to child: {:?}", status);
        let mut msg = serde_json::to_string(&status)?;
        msg += "\n";
        stdin.write_all(msg.as_bytes())?;
        stdin.flush()?;
        debug!("notified child");
        Ok(())
    } else {
        bail!("stdin of child is not piped");
    }
}

fn zmq_thread(_socket: &str, mut decap: Child, config: &mut Config) -> Result<()> {
    if !config.danger_disable_seccomp_security {
        sandbox::zmq_stage1()
            .context("sandbox zmq_stage1 failed")?;
    }

    let mut status = None;

    // resolve gid before running chroot
    config.daemon.resolve_gid()?;

    if !config.danger_disable_seccomp_security {
        sandbox::zmq_stage2()
            .context("sandbox zmq_stage2 failed")?;
    }

    // let mut server = Server::bind(socket, config)?;
    // TODO: this only works when chrooted:
    let mut server = Server::bind("ipc:///snail.sock", config)?;

    if !config.danger_disable_seccomp_security {
        sandbox::zmq_stage3()
            .context("sandbox zmq_stage3 failed")?;
    }

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
                        let network = NetworkStatus::new(event.ssid, net);
                        status = Some(network.clone());
                        send_to_child(&mut decap, network)?;
                    },
                    Some(UpdateMessage::Reboot(net)) => {
                        info!("successful dhcp reboot");
                        let network = NetworkStatus::new(event.ssid, net);
                        status = Some(network.clone());
                        send_to_child(&mut decap, network)?;
                    },
                    Some(UpdateMessage::Renew(_net)) => {
                        // ignore
                        debug!("dhcp renewed");
                    },
                    Some(UpdateMessage::NoCarrier) => {
                        info!("carrier lost");
                        status = None;
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
                CtlReply::Status(status.clone())
            },
            CtlRequest::SetStatus(update) => {
                status = update;
                CtlReply::Ack
            },
        };

        server.reply(&reply)?;
    }
}

fn notify_daemon() -> Result<()> {
    let event = dhcp::read_dhcp_env()?;
    debug!("event: {:?}", event);

    // this can prevent dhcpcd from shutting down if the zmq thread already stopped
    if event.message == Some(dhcp::UpdateMessage::Stopped) {
        return Ok(());
    }

    let config = config::read_from(config::PATH)
                    .context("failed to load config")?;
    debug!("config: {:?}", config);

    let mut client = Client::connect(&config.daemon.socket)?;
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

            let mut config = config::read_from(config::PATH)
                                .context("failed to load config")?;
            debug!("config: {:?}", config);

            let socket = args.socket.unwrap_or(config.daemon.socket.clone());

            match args.subcommand {
                Some(SubCommand::Start(args)) => {
                    let myself = {
                        let h = env::current_exe().unwrap();
                        h.to_str().unwrap().to_string()
                    };

                    // TODO: log level isn't forwarded to children

                    let _dhcp_child = Command::new(&myself)
                        .args(&["dhcp", &args.interface])
                        .stdin(Stdio::null())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()?;

                    let decap_child = Command::new(&myself)
                        .args(&["decap"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .spawn()?;

                    zmq_thread(&socket, decap_child, &mut config)
                },
                Some(SubCommand::Dhcp(args)) => {
                    let hook = {
                        let h = env::current_exe().unwrap();
                        h.to_str().unwrap().to_string()
                    };

                    dhcp_thread(&args.interface, &hook)
                },
                Some(SubCommand::Decap) => {
                    decap_thread(&socket, &config)
                },
                None => {
                    error!("dhcp event expected but not found");
                    Ok(())
                },
            }
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
