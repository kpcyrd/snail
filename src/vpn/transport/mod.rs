use errors::Result;

use vpn::wire::Packet;

use std::net::SocketAddr;

pub mod udp;


pub trait ClientTransport {
    fn recv(&self) -> Result<Packet>;

    fn send(&self, pkt: &Packet) -> Result<()>;
}

pub trait ServerTransport {
    fn recv_from(&self) -> Result<(Packet, SocketAddr)>;

    fn send_to(&self, pkt: &Packet, dst: &SocketAddr) -> Result<()>;
}
