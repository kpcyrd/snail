use errors::Result;

use std::net::SocketAddr;

pub mod udp;


pub trait ClientTransport {
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn send(&mut self, buf: &[u8]) -> Result<usize>;
}

pub trait ServerTransport {
    fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr)>;

    fn send_to(&mut self, buf: &[u8], dst: &SocketAddr) -> Result<usize>;
}
