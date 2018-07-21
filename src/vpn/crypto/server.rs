use errors::{Result, Error};

use vpn::crypto::{Handshake, Channel};
use vpn::transport::ServerTransport;
use vpn::wire::Packet;

use base64;

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
        let pkt = Packet::make_handshake(msg);
        self.transport.send_to(&pkt, dst)?;
        Ok(())
    }

    pub fn recv_from(&mut self) -> Result<SocketAddr> {
        let (pkt, src) = self.transport.recv_from()?;

        let pkt = pkt.handshake()?;
        match self.handshake.insert(&pkt.bytes) {
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
        let (pkt, src) = self.transport.recv_from()?;
        let msg = self.channel.decrypt(&pkt)?;
        Ok((msg, src))
    }

    pub fn send_to(&mut self, buf: &[u8], dst: &SocketAddr) -> Result<()> {
        let pkt = self.channel.encrypt(buf)?;
        self.transport.send_to(&pkt, dst)?;
        Ok(())
    }

    #[inline]
    pub fn remote_pubkey(&self) -> Result<Vec<u8>> {
        self.channel.remote_pubkey()
    }
}
