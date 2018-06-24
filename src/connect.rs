use std::io;
use std::io::prelude::*;
use std::thread;
use std::time::Duration;
use std::net::{SocketAddr, TcpStream};
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd};

use dns::DnsResolver;
use errors::Result;


#[inline]
pub fn from(fd: RawFd) -> TcpStream {
    unsafe { TcpStream::from_raw_fd(fd) }
}

pub fn pipe(mut r: impl Read, mut w: impl Write) {
    let mut buf = [0; 1024];

    loop {
        let n = r.read(&mut buf).expect("read");

        // read until EOF
        if n == 0 {
            break;
        }

        w.write(&buf[..n]).expect("write");
        w.flush().expect("flush");
    }
}

pub fn connect<R: DnsResolver>(resolver: R, host: &str, port: u16) -> Result<()> {
    let records = resolver.resolve(host)?;

    for ip in records {
        let addr = SocketAddr::new(ip, port);
        debug!("connecting to {}", addr);

        if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
            info!("connection opened: {}", addr);
            let f = stream.as_raw_fd();

            let fd = f.clone();
            let t1 = thread::spawn(move || pipe(from(fd), io::stdout()));
            let fd = f.clone();
            let t2 = thread::spawn(move || pipe(io::stdin(), from(fd)));

            t1.join().unwrap();
            t2.join().unwrap();

            break;
        } else {
            debug!("connection failed");
        }
    }

    Ok(())
}
