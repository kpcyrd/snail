use errors::Result;

use byteorder::{BigEndian, WriteBytesExt, ReadBytesExt};
use nom::IResult;


fn header(input: &[u8]) -> IResult<&[u8], Header> {
    let (remaining, options) = header_options(input)?;
    if options.handshake {
        handshake(remaining, options.stage)
    } else {
        transport(remaining, options.truncated)
    }
}

fn header_options(input: &[u8]) -> IResult<&[u8], HeaderOptions> {
    do_parse!(input,
        options: bits!(tuple!(
            map!(take_bits!(u8, 1), |x| x > 0), // handshake vs transport
            take_bits!(u8, 2), // handshake stage
            take_bits!(u8, 2), // unused
            take_bits!(u8, 3)) // number of truncated nonce bytes
        ) >>
        ({
            HeaderOptions {
                handshake: options.0,
                stage: options.1,
                truncated: options.3,
            }
        })
    )
}

fn handshake(input: &[u8], _stage: u8) -> IResult<&[u8], Header> {
    do_parse!(input,
        ({
            Header::Handshake
        })
    )
}

fn transport(input: &[u8], truncated: u8) -> IResult<&[u8], Header> {
    do_parse!(input,
        nonce: take!(8 - truncated)     >>
        ({
            let truncated = truncated as usize;

            // parse truncated u64 into u64
            let mut buf = [0u8; 8];
            for (i, x) in nonce.iter().enumerate() {
                buf[truncated+i] = *x;
            }
            let nonce = (&buf[..]).read_u64::<BigEndian>().unwrap();

            Header::Transport(nonce)
        })
    )
}

#[derive(Debug)]
pub enum Header {
    Handshake,
    Transport(u64),
}

impl Header {
    pub fn packet(self, bytes: Vec<u8>) -> Packet {
        match self {
            Header::Handshake => Packet::make_handshake(bytes),
            Header::Transport(nonce) => Packet::make_transport(nonce, bytes),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct HeaderOptions {
    handshake: bool,
    stage: u8,
    truncated: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Packet {
    Handshake(Handshake),
    Transport(Transport),
}

impl Packet {
    #[inline]
    pub fn make_handshake(bytes: Vec<u8>) -> Packet {
        Packet::Handshake(Handshake {
            bytes,
        })
    }

    #[inline]
    pub fn make_transport(nonce: u64, bytes: Vec<u8>) -> Packet {
        Packet::Transport(Transport {
            nonce,
            bytes,
        })
    }

    pub fn handshake(&self) -> Result<&Handshake> {
        match *self {
            Packet::Handshake(ref handshake) => Ok(&handshake),
            _ => bail!("not a handshake packet"),
        }
    }

    pub fn transport(&self) -> Result<&Transport> {
        match *self {
            Packet::Transport(ref transport) => Ok(&transport),
            _ => bail!("not a transport packet"),
        }
    }

    #[inline]
    pub fn pack_nonce<'a>(buf: &'a mut Vec<u8>, nonce: u64) -> (u8, &'a [u8]) {
        buf.write_u64::<BigEndian>(nonce).unwrap();
        for x in 0..7 {
            if buf[x] != 0 {
                return (x as u8, &buf[x..]);
            }
        }
        return (7, &buf[7..]);
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match *self {
            Packet::Handshake(ref pkt) => {
                buf.push(0x80);
                buf.extend(&pkt.bytes);
            },
            Packet::Transport(ref pkt) => {
                let mut options = 0;

                // compress nonce
                let mut nonce = Vec::with_capacity(8);
                let (truncated, nonce) = Self::pack_nonce(&mut nonce, pkt.nonce);
                options ^= truncated;

                buf.push(options);
                buf.extend(nonce);
                buf.extend(&pkt.bytes);
            },
        };

        buf
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Handshake {
    // TODO: consider adding stage to packet
    pub bytes: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transport {
    pub nonce: u64,
    pub bytes: Vec<u8>,
}

pub fn packet(input: &[u8]) -> Result<Packet> {
    if let Ok((remaining, header)) = header(input) {
        Ok(header.packet(remaining.to_vec()))
    } else {
        bail!("could not parse packet")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkt_handshake() {
        let orig = Packet::make_handshake(b"ohai\n".to_vec());

        let pkt = packet(&orig.as_bytes()).expect("failed to parse packet");
        assert_eq!(pkt, orig);

        let pkt = pkt.handshake().expect("not a handshake packet");
        assert_eq!(&pkt.bytes, b"ohai\n");
    }

    #[test]
    fn test_pkt_transport() {
        let orig = Packet::make_transport(0x1122334455667788, b"ohai\n".to_vec());

        let pkt = packet(&orig.as_bytes()).expect("failed to parse packet");
        assert_eq!(pkt, orig);

        let pkt = pkt.transport().expect("not a transport packet");
        assert_eq!(pkt.nonce, 0x1122334455667788);
        assert_eq!(&pkt.bytes, b"ohai\n");
    }

    #[test]
    fn test_pkt_transport_short_nonce() {
        let orig = Packet::make_transport(0xffff, b"ohai\n".to_vec());

        let pkt = packet(&orig.as_bytes()).expect("failed to parse packet");
        assert_eq!(pkt, orig);

        let pkt = pkt.transport().expect("not a transport packet");
        assert_eq!(pkt.nonce, 0xffff);
        assert_eq!(&pkt.bytes, b"ohai\n");
    }

    #[test]
    fn test_pkt_transport_zero_nonce() {
        let orig = Packet::make_transport(0x00, b"ohai\n".to_vec());

        let pkt = packet(&orig.as_bytes()).expect("failed to parse packet");
        assert_eq!(pkt, orig);

        let pkt = pkt.transport().expect("not a transport packet");
        assert_eq!(pkt.nonce, 0x00);
        assert_eq!(&pkt.bytes, b"ohai\n");
    }
}
