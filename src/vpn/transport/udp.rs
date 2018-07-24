use errors::Result;
use vpn::transport::{ClientTransport, ServerTransport};
use vpn::wire::{self, Packet};

use std::net::UdpSocket;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;


#[derive(Debug)]
pub struct UdpClient {
    socket: UdpSocket,
}

impl UdpClient {
    pub fn connect<T: ToSocketAddrs>(remote: T) -> Result<UdpClient> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(remote)?;

        Ok(UdpClient {
            socket,
        })
    }
}

impl ClientTransport for UdpClient {
    fn recv(&self) -> Result<Packet> {
        let mut buf = [0; 1600]; // TODO: adjust size

        let n = self.socket.recv(&mut buf)?;
        let buf = &buf[..n];

        debug!("recv(udp): {:?}", buf);
        wire::packet(&buf)
    }

    fn send(&self, pkt: &Packet) -> Result<()> {
        let buf = pkt.as_bytes();
        debug!("send(udp): {:?}", buf);
        self.socket.send(&buf)?;
        Ok(())
    }
}

#[derive(Debug)]
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
    fn recv_from(&self) -> Result<(Packet, SocketAddr)> {
        let mut buf = [0; 1600]; // TODO: adjust size

        let (n, src) = self.socket.recv_from(&mut buf)?;
        let buf = &buf[..n];
        debug!("[{}] recv(udp): {:?}", src, buf);

        let pkt = wire::packet(&buf)?;
        Ok((pkt, src))
    }

    fn send_to(&self, pkt: &Packet, dst: &SocketAddr) -> Result<()> {
        let buf = pkt.as_bytes();
        debug!("[{}] send(udp): {:?}", dst, buf);
        self.socket.send_to(&buf, dst)?;
        Ok(())
    }
}
