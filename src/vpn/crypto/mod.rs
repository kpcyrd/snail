use errors::{Result, Error, ResultExt};

use vpn::wire::Packet;

use snow::{self, Builder};
use snow::params::NoiseParams;

pub mod client;
pub use self::client::ClientHandshake;
pub mod server;
pub use self::server::ServerHandshake;


pub fn gen_key() -> Result<(Vec<u8>, Vec<u8>)> {
    Builder::new(Handshake::gen_params())
        .generate_keypair()
        .map_err(Error::from)
        .map(|keypair| (keypair.public, keypair.private))
}

#[derive(Debug)]
pub enum Session {
    Handshake(Handshake),
    Channel(Channel),
}

#[derive(Debug)]
pub struct Handshake {
    noise: snow::Session,
}

impl Handshake {
    fn gen_params() -> NoiseParams {
        // TODO: consider using Noise_XX if there are advantages
        // TODO: consider using a +psk mode for DoS protection (if needed)
        "Noise_XK_25519_ChaChaPoly_BLAKE2s".parse::<NoiseParams>().expect("noise parameter is invalid")
    }

    fn new<'a>(local_privkey: &'a [u8]) -> Builder<'a> {
        let params = Self::gen_params();
        let builder = Builder::new(params.clone());

        builder
            .local_private_key(&local_privkey)
    }

    pub fn responder(local_privkey: &[u8]) -> Result<Handshake> {
        let noise = Handshake::new(&local_privkey)
                        .build_responder()?;

        Ok(Handshake {
            noise,
        })
    }

    pub fn initiator(remote_pubkey: &[u8], local_privkey: &[u8]) -> Result<Handshake> {
        let noise = Handshake::new(local_privkey)
                        .remote_public_key(remote_pubkey)
                        .build_initiator()?;

        Ok(Handshake {
            noise,
        })
    }

    pub fn insert(&mut self, cipher: &[u8]) -> Result<()> {
        let mut buf = vec![0u8; 65535];

        let _n = self.noise.read_message(cipher, &mut buf)
                        .context("failed to read noise")?;

        Ok(())
    }

    pub fn take(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 65535];

        let n = self.noise.write_message(&[0u8; 0], &mut buf)
                        .context("failed to write noise")?;

        Ok(buf[..n].to_vec())
    }

    pub fn take_packet(&mut self) -> Result<Packet> {
        let pkt = self.take()?;
        let pkt = Packet::make_handshake(pkt);
        Ok(pkt)
    }

    #[inline]
    pub fn is_handshake_finished(&self) -> bool {
        self.noise.is_handshake_finished()
    }

    pub fn channel(self) -> Result<Channel> {
        let noise = self.noise.into_transport_mode()
                            .context("could not switch into transport mode")?;
        Ok(Channel {
            noise,
        })
    }
}

#[derive(Debug)]
pub struct Channel {
    noise: snow::Session,
}

impl Channel {
    pub fn remote_pubkey(&self) -> Result<Vec<u8>> {
        self.noise.get_remote_static()
            .map(|p| Vec::from(p))
            .ok_or(format_err!("remote did not send longterm pubkey"))
    }

    pub fn decrypt(&mut self, packet: &Packet) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 65535];

        let packet = packet.transport()?;
        self.noise.set_receiving_nonce(packet.nonce)?;
        let n = self.noise.read_message(&packet.bytes, &mut buf)
                        .context("failed to read noise")?;

        Ok(buf[..n].to_vec())
    }

    pub fn encrypt(&mut self, msg: &[u8]) -> Result<Packet> {
        let mut buf = vec![0u8; 65535];

        let nonce = self.noise.sending_nonce()?;
        let n = self.noise.write_message(msg, &mut buf)
                        .context("failed to write noise")?;

        Ok(Packet::make_transport(nonce, buf[..n].to_vec()))
    }

    // TODO: rekey
}
