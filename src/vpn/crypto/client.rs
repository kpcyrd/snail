use errors::Result;

use vpn::crypto::{Handshake, Channel};
use vpn::transport::ClientTransport;

use base64;
// use nom;


pub struct ClientHandshake<T: ClientTransport> {
    handshake: Handshake,
    transport: T,
}

impl<T: ClientTransport> ClientHandshake<T> {
    pub fn initiator(transport: T, server_pubkey: &str, client_privkey: &str) -> Result<ClientHandshake<T>> {
        let server_pubkey = base64::decode(&server_pubkey)?;
        let client_privkey = base64::decode(&client_privkey)?;

        let handshake = Handshake::initiator(&server_pubkey, &client_privkey)?;

        Ok(ClientHandshake {
            handshake,
            transport,
        })
    }

    pub fn send(&mut self) -> Result<()> {
        let msg = self.handshake.take()?;
        self.transport.send(&msg)?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<()> {
        let mut buf = [0; 1600]; // TODO: adjust size

        let amt = self.transport.recv(&mut buf)?;
        self.handshake.insert(&mut buf[..amt])?;
        Ok(())
    }

    #[inline]
    pub fn is_handshake_finished(&self) -> bool {
        self.handshake.is_handshake_finished()
    }

    pub fn channel(self) -> Result<ClientChannel<T>> {
        let channel = self.handshake.channel()?;
        let transport = self.transport;

        Ok(ClientChannel {
            channel,
            transport,
        })
    }
}

pub struct ClientChannel<T: ClientTransport> {
    channel: Channel,
    transport: T,
}

impl<T: ClientTransport> ClientChannel<T> {
    pub fn recv(&mut self) -> Result<Vec<u8>> {
        let mut encrypted = vec![0u8; 65535];
        let amt = self.transport.recv(&mut encrypted)?;
        let msg = self.channel.decrypt(&encrypted[..amt])?;
        Ok(msg)
    }

    pub fn send(&mut self, buf: &[u8]) -> Result<()> {
        let msg = self.channel.encrypt(buf)?;
        self.transport.send(&msg)?;
        Ok(())
    }
}
