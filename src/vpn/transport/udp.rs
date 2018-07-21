use errors::{Result, Error};
use vpn::transport::{ClientTransport, ServerTransport};

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
    fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.socket.recv(buf)?;
        let buf = &buf[..n];
        debug!("recv(udp): {:?}", buf);
        Ok(n)
    }

    fn send(&mut self, buf: &[u8]) -> Result<usize> {
        debug!("send(udp): {:?}", buf);
        self.socket.send(buf)
            .map_err(Error::from)
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
    fn recv_from(&mut self, buf: &mut [u8]) -> Result<(usize, SocketAddr)> {
        let (n, src) = self.socket.recv_from(buf)?;
        let buf = &buf[..n];
        debug!("[{}] recv(udp): {:?}", src, buf);
        Ok((n, src))
    }

    fn send_to(&mut self, buf: &[u8], dst: &SocketAddr) -> Result<usize> {
        debug!("[{}] send(udp): {:?}", dst, buf);
        self.socket.send_to(buf, dst)
            .map_err(Error::from)
    }
}
