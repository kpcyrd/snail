// TODO: this whole file is a temporary solution until we don't need child processes to configure interfaces
use errors::{Result, ResultExt};

use vpn;

use cidr::Ipv4Inet;
use serde_json;

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::process::{Command, Child, Stdio};
use std::net::Ipv4Addr;


#[derive(Debug, Serialize, Deserialize)]
pub enum IfconfigCommand {
    Ipconfig((String, Ipv4Inet)),
    GetRoute(Ipv4Addr),
    AddRoute((Ipv4Inet, Ipv4Addr)),
}

#[derive(Debug)]
pub struct IfconfigChild {
    child: Child,
}

impl IfconfigChild {
    // TODO: consider forcing interface in IfconfigChild
    pub fn ipconfig(&mut self, interface: &str, addr: &Ipv4Inet) -> Result<()> {
        self.send_to_child(IfconfigCommand::Ipconfig((interface.to_string(), addr.clone())))
    }

    pub fn get_route(&mut self, target: &Ipv4Addr) -> Result<Ipv4Addr> {
        self.send_to_child(IfconfigCommand::GetRoute(target.clone()))?;
        self.read_reply()
    }

    pub fn add_route(&mut self, range: &Ipv4Inet, gateway: &Ipv4Addr) -> Result<()> {
        self.send_to_child(IfconfigCommand::AddRoute((range.clone(), gateway.clone())))
    }

    pub fn tunnel_all_traffic(&mut self, gateway: &Ipv4Addr) -> Result<()> {
        self.add_route(&"0.0.0.0/1".parse()?, &gateway)?;
        self.add_route(&"128.0.0.0/1".parse()?, &gateway)?;
        Ok(())
    }

    fn send_to_child(&mut self, command: IfconfigCommand) -> Result<()> {
        if let Some(stdin) = &mut self.child.stdin {
            debug!("sending to child: {:?}", command);
            let mut msg = serde_json::to_string(&command)?;
            msg += "\n";
            stdin.write_all(msg.as_bytes())?;
            stdin.flush()?;
            debug!("notified child");
            Ok(())
        } else {
            bail!("stdin of child is not piped");
        }
    }

    fn read_reply(&mut self) -> Result<Ipv4Addr> {
        if let Some(stdout) = &mut self.child.stdout {
            let buf = &mut [0u8; 512];
            let n = stdout.read(buf)?;

            let reply = serde_json::from_slice(&buf[..n])?;
            Ok(reply)
        } else {
            bail!("stdout of child is not piped");
        }
    }
}

pub fn spawn() -> Result<IfconfigChild> {
    use std::env;

    let myself = {
        let h = env::current_exe().unwrap();
        h.to_str().unwrap().to_string()
    };

    // TODO: log level isn't forwarded to children
    let child = Command::new(&myself)
        .args(&["ifconfig"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(IfconfigChild {
        child
    })
}

fn send_reply(addr: Ipv4Addr) -> Result<()> {
    let msg = serde_json::to_string(&addr)?;
    println!("{}", msg);
    Ok(())
}

pub fn run() -> Result<()> {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    for msg in reader.lines() {
        debug!("got command: {:?}", msg);
        let msg = msg?;
        let msg = serde_json::from_str::<IfconfigCommand>(&msg)?;
        debug!("command: {:?}", msg);

        match msg {
            IfconfigCommand::Ipconfig((interface, addr)) => {
                vpn::ipconfig(&interface, &addr)
                    .context("failed to set ip")?;
            },
            IfconfigCommand::GetRoute(target) => {
                let route = vpn::get_route(&target)
                    .context("failed to get route")?;
                send_reply(route)?;
            },
            IfconfigCommand::AddRoute((range, gateway)) => {
                vpn::add_route(&range, &gateway)
                    .context("failed to set route")?;
            },
        }
    }

    Ok(())
}
