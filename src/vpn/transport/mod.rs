use errors::Result;

use vpn::wire::Packet;

use std::net::SocketAddr;

pub mod udp;


pub trait ClientTransport {
    fn recv(&mut self) -> Result<Packet>;

    fn send(&mut self, pkt: &Packet) -> Result<()>;
}

pub trait ServerTransport {
    fn recv_from(&mut self) -> Result<(Packet, SocketAddr)>;

    fn send_to(&mut self, pkt: &Packet, dst: &SocketAddr) -> Result<()>;
}
