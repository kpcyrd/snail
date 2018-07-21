use errors::{Result, Error};

use vpn::crypto::{Handshake, Channel};
use vpn::transport::ServerTransport;

use base64;
// use nom;

use std::net::SocketAddr;


pub struct ServerHandshake<T: ServerTransport> {
    handshake: Handshake,
    transport: T,
}

impl<T: ServerTransport> ServerHandshake<T> {
    pub fn responder(transport: T, server_privkey: &str) -> Result<ServerHandshake<T>> {
        let server_privkey = base64::decode(&server_privkey)?;

        let handshake = Handshake::responder(&server_privkey)?;

        Ok(ServerHandshake {
            handshake,
            transport,
        })
    }

    pub fn send_to(&mut self, dst: &SocketAddr) -> Result<()> {
        let msg = self.handshake.take()?;
        self.transport.send_to(&msg, dst)?;
        Ok(())
    }

    pub fn recv_from(&mut self) -> Result<SocketAddr> {
        let mut buf = [0; 1600]; // TODO: adjust size

        let (amt, src) = self.transport.recv_from(&mut buf)?;

        match self.handshake.insert(&mut buf[..amt]) {
            Ok(_) => Ok(src),
            Err(e) => {
                warn!("[{}] client sent bad data during handshake: {:?}", src, e);
                Err(Error::from(e))
            },
        }
    }

    #[inline]
    pub fn is_handshake_finished(&self) -> bool {
        self.handshake.is_handshake_finished()
    }

    pub fn channel(self) -> Result<ServerChannel<T>> {
        let channel = self.handshake.channel()?;
        let transport = self.transport;

        Ok(ServerChannel {
            channel,
            transport,
        })
    }
}

pub struct ServerChannel<T: ServerTransport> {
    channel: Channel,
    transport: T,
}

impl<T: ServerTransport> ServerChannel<T> {
    pub fn recv_from(&mut self) -> Result<(Vec<u8>, SocketAddr)> {
        let mut encrypted = vec![0u8; 65535];
        let (amt, src) = self.transport.recv_from(&mut encrypted)?;
        let msg = self.channel.decrypt(&encrypted[..amt])?;
        Ok((msg, src))
    }

    pub fn send_to(&mut self, buf: &[u8], dst: &SocketAddr) -> Result<()> {
        let msg = self.channel.encrypt(buf)?;
        self.transport.send_to(&msg, dst)?;
        Ok(())
    }

    #[inline]
    pub fn remote_pubkey(&self) -> Result<Vec<u8>> {
        self.channel.remote_pubkey()
    }
}
