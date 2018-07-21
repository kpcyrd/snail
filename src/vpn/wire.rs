use errors::Result;

use byteorder::{BigEndian, WriteBytesExt};
use nom::{IResult, be_u8, be_u64};


fn header(input: &[u8]) -> IResult<&[u8], Header> {
    do_parse!(input,
        header: switch!(be_u8,
            0x00 => call!(transport) |
            0x01 => call!(handshake)
        ) >>
        ({
            header
        })
    )
}

fn handshake(input: &[u8]) -> IResult<&[u8], Header> {
    do_parse!(input,
        ({
            Header::Handshake
        })
    )
}

fn transport(input: &[u8]) -> IResult<&[u8], Header> {
    do_parse!(input,
        nonce: be_u64       >>
        ({
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

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match *self {
            Packet::Handshake(ref pkt) => {
                buf.push(0x01);
                buf.extend(&pkt.bytes);
            },
            Packet::Transport(ref pkt) => {
                buf.push(0x00);
                buf.write_u64::<BigEndian>(pkt.nonce).unwrap();
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
}
