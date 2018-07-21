use errors::Result;
use vpn::transport::{ClientTransport, ServerTransport};
use vpn::wire::{self, Packet};

use std::net::UdpSocket;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;


pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    pub fn connect<T: ToSocketAddrs, U: ToSocketAddrs>(bind: T, remote: U) -> Result<UdpClient> {
        let socket = UdpSocket::bind(bind)?;
        socket.connect(remote)?;

        Ok(UdpClient {
            socket,
        })
    }
}

impl ClientTransport for UdpClient {
    fn recv(&mut self) -> Result<Packet> {
        let mut buf = [0; 1600]; // TODO: adjust size

        let n = self.socket.recv(&mut buf)?;
        let buf = &buf[..n];

        debug!("recv(udp): {:?}", buf);
        wire::packet(&buf)
    }

    fn send(&mut self, pkt: &Packet) -> Result<()> {
        let buf = pkt.as_bytes();
        debug!("send(udp): {:?}", buf);
        self.socket.send(&buf)?;
        Ok(())
    }
}

pub struct UdpServer {
    socket: UdpSocket,
}

impl UdpServer {
    pub fn bind<T: ToSocketAddrs>(bind: T) -> Result<UdpServer> {
        let socket = UdpSocket::bind(bind)?;

        Ok(UdpServer {
            socket,
        })
    }
}

impl ServerTransport for UdpServer {
    fn recv_from(&mut self) -> Result<(Packet, SocketAddr)> {
        let mut buf = [0; 1600]; // TODO: adjust size

        let (n, src) = self.socket.recv_from(&mut buf)?;
        let buf = &buf[..n];
        debug!("[{}] recv(udp): {:?}", src, buf);

        let pkt = wire::packet(&buf)?;
        Ok((pkt, src))
    }

    fn send_to(&mut self, pkt: &Packet, dst: &SocketAddr) -> Result<()> {
        let buf = pkt.as_bytes();
        debug!("[{}] send(udp): {:?}", dst, buf);
        self.socket.send_to(&buf, dst)?;
        Ok(())
    }
}
