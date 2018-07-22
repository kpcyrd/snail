use errors::Result;

use vpn::wire::Packet;

use snow::{self, Builder};
use snow::resolvers::{CryptoResolver, DefaultResolver};
use snow::params::{NoiseParams, DHChoice};

pub mod client;
pub use self::client::ClientHandshake;
pub mod server;
pub use self::server::ServerHandshake;


pub fn gen_key() -> Result<(Vec<u8>, Vec<u8>)> {
    let resolver = DefaultResolver;
    let dh = DHChoice::Curve25519;

    let mut rng = resolver.resolve_rng().unwrap();
    let mut dh = resolver.resolve_dh(&dh).unwrap();

    let mut public = vec![0u8; dh.pub_len()];
    let mut private = vec![0u8; dh.priv_len()];
    dh.generate(&mut *rng);

    public[..dh.pub_len()].copy_from_slice(dh.pubkey());
    private[..dh.priv_len()].copy_from_slice(dh.privkey());
    Ok((public, private))
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
        let noise = match Handshake::new(&local_privkey)
                .build_responder() {
            Ok(noise) => noise,
            Err(e) => bail!("failed to build responder: {:?}", e),
        };

        Ok(Handshake {
            noise,
        })
    }

    pub fn initiator(remote_pubkey: &[u8], local_privkey: &[u8]) -> Result<Handshake> {
        let noise = match Handshake::new(local_privkey)
                .remote_public_key(remote_pubkey)
                .build_initiator() {
            Ok(noise) => noise,
            Err(e) => bail!("failed to build initiator: {:?}", e),
        };

        Ok(Handshake {
            noise,
        })
    }

    pub fn insert(&mut self, cipher: &[u8]) -> Result<()> {
        let mut buf = vec![0u8; 65535];

        let _n = match self.noise.read_message(cipher, &mut buf) {
            Ok(n) => n,
            Err(e) => bail!("failed to read noise: {:?}", e),
        };

        Ok(())
    }

    pub fn take(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; 65535];

        let n = match self.noise.write_message(&[0u8; 0], &mut buf) {
            Ok(n) => n,
            Err(e) => bail!("failed to write noise: {:?}", e),
        };

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
        match self.noise.into_transport_mode() {
            Ok(noise) => Ok(Channel {
                noise,
            }),
            Err(e) => bail!("could not switch into transport mode: {:?}", e),
        }
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
        let n = match self.noise.read_message(&packet.bytes, &mut buf) {
            Ok(n) => n,
            Err(e) => bail!("failed to read noise: {:?}", e),
        };

        Ok(buf[..n].to_vec())
    }

    pub fn encrypt(&mut self, msg: &[u8]) -> Result<Packet> {
        let mut buf = vec![0u8; 65535];

        let nonce = self.noise.sending_nonce()?;
        let n = match self.noise.write_message(msg, &mut buf) {
            Ok(n) => n,
            Err(e) => bail!("failed to write noise: {:?}", e),
        };

        Ok(Packet::make_transport(nonce, buf[..n].to_vec()))
    }

    // TODO: rekey
}
