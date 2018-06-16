// TODO: socket permissions are currently disabled
use zmq;
use serde_json;

use ::Result;
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use dhcp::NetworkUpdate;
use wifi::NetworkStatus;


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
    pub fn bind(url: &str) -> Result<Server> {
        let ctx = zmq::Context::new();
        let socket = ctx.socket(zmq::REP)?;

        socket.bind(url)?;

        // fix permissions
        if url.starts_with("ipc://") {
            // TODO: write a proper solution
            // let perms = Permissions::from_mode(0o770);
            // TODO: FIXME: socket perissions are fully disabled
            let perms = Permissions::from_mode(0o777);
            fs::set_permissions(&url[6..], perms)?;
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
