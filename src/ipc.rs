use zmq;
use serde_json;
use nix;
use nix::unistd::Gid;
use users;

use config::Config;
use dhcp::NetworkUpdate;
use errors::Result;
use wifi::NetworkStatus;

use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;


pub const SOCKET: &str = "ipc:///run/snail.sock";

#[derive(Debug, Serialize, Deserialize)]
pub enum CtlRequest {
    Ping,
    DhcpEvent(NetworkUpdate),
    StatusRequest,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CtlReply {
    Pong,
    Ack,
    Status(Option<NetworkStatus>),
}


pub struct Server {
    #[allow(dead_code)]
    ctx: zmq::Context,
    socket: zmq::Socket,
}

impl Server {
    pub fn bind(url: &str, config: &Config) -> Result<Server> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REP)?;

        socket.bind(url)?;

        // fix permissions
        if url.starts_with("ipc://") {
            // TODO: write a proper solution
            let path = &url[6..];

            let perms = Permissions::from_mode(0o770);
            fs::set_permissions(&path, perms)?;

            if let Some(group) = &config.daemon.socket_group {
                if let Some(gid) = users::get_group_by_name(&group) {
                    let gid = Gid::from_raw(gid.gid());
                    nix::unistd::chown(path, None, Some(gid))?;
                    info!("socket group has been set to {:?} ({})", group, gid);
                } else {
                    bail!("group not found");
                }
            }
        }

        Ok(Server {
            ctx,
            socket,
        })
    }

    pub fn recv(&mut self) -> Result<CtlRequest> {
        let bytes = self.socket.recv_msg(0)?;
        let req = serde_json::from_str(bytes.as_str().unwrap())?;
        debug!("ctl(req): {:?}", req);
        Ok(req)
    }

    pub fn reply(&mut self, rep: &CtlReply) -> Result<()> {
        debug!("ctl(rep): {:?}", rep);
        let bytes = serde_json::to_string(rep)?;
        self.socket.send(bytes.as_bytes(), 0)?;
        Ok(())
    }
}

pub struct Client {
    #[allow(dead_code)]
    ctx: zmq::Context,
    socket: zmq::Socket,
}

impl Client {
    pub fn connect(url: &str) -> Result<Client> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REQ)?;

        socket.connect(url)?;

        Ok(Client {
            ctx,
            socket,
        })
    }

    pub fn send(&mut self, req: &CtlRequest) -> Result<CtlReply> {
        debug!("ctl(req): {:?}", req);

        let bytes = serde_json::to_string(req)?;
        self.socket.send(bytes.as_bytes(), 0)?;

        let bytes = self.socket.recv_msg(0)?;
        let rep = serde_json::from_str(bytes.as_str().unwrap())?;

        debug!("ctl(rep): {:?}", rep);
        Ok(rep)
    }

    pub fn status(&mut self) -> Result<Option<NetworkStatus>> {
        if let CtlReply::Status(status) = self.send(&CtlRequest::StatusRequest)? {
            Ok(status)
        } else {
            bail!("Wrong ctl reply");
        }
    }
}
