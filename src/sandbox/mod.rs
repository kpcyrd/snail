use errors::Result;

use nix;

use std::env;

pub mod seccomp;
pub mod syscalls;

pub const CHROOT: &str = "/run/snail";


pub fn chroot_socket_path(socket: &str, chroot: &str) -> Result<String> {
    if !socket.starts_with("ipc://") {
        return Ok(socket.to_string());
    }

    let path = &socket[6..];

    let mut chroot = chroot.to_string();
    if !chroot.ends_with("/") {
        chroot += "/";
    }

    if !path.starts_with(&chroot) {
        bail!("socket path is outside of chroot");
    }

    // include the previous slash
    let path = &path[chroot.len()-1..];

    Ok(format!("ipc://{}", path))
}

pub fn chroot(path: &str) -> Result<()> {
    nix::unistd::chroot(path)?;
    env::set_current_dir("/")?;
    Ok(())
}

pub fn decap_stage1() -> Result<()> {
    seccomp::decap_stage1()?;
    info!("decap_stage 1/2 enabled");
    Ok(())
}

pub fn decap_stage2() -> Result<()> {
    chroot("/run/snail")?;
    info!("decap_stage 2/3 enabled");
    Ok(())
}

pub fn decap_stage3() -> Result<()> {
    seccomp::decap_stage3()?;
    info!("decap_stage 3/3 enabled");
    Ok(())
}

pub fn zmq_stage1() -> Result<()> {
    seccomp::zmq_stage1()?;
    info!("zmq_stage 1/2 enabled");
    Ok(())
}

pub fn zmq_stage2() -> Result<()> {
    chroot("/run/snail")?;
    info!("zmq_stage 2/3 enabled");
    Ok(())
}

pub fn zmq_stage3() -> Result<()> {
    seccomp::zmq_stage3()?;
    info!("zmq_stage 3/3 enabled");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_chroot_socket_path() {
        let x = chroot_socket_path("ipc:///run/snail/snail.sock", "/run/snail").expect("chroot_socket_path");
        assert_eq!(&x, "ipc:///snail.sock");
    }

    #[test]
    fn verify_chroot_socket_outside_of_chroot() {
        let x = chroot_socket_path("ipc:///run/snail/snail.sock", "/var/empty");
        assert!(x.is_err());
    }

    #[test]
    fn verify_chroot_socket_run() {
        let x = chroot_socket_path("ipc:///run/snail/snail.sock", "/run").expect("chroot_socket_path");
        assert_eq!(&x, "ipc:///snail/snail.sock");
    }

    #[test]
    fn verify_chroot_socket_root() {
        let x = chroot_socket_path("ipc:///run/snail/snail.sock", "/").expect("chroot_socket_path");
        assert_eq!(&x, "ipc:///run/snail/snail.sock");
    }

    #[test]
    fn verify_chroot_socket_not_ipc() {
        let x = chroot_socket_path("tcp://127.0.0.1", "/").expect("chroot_socket_path");
        assert_eq!(&x, "tcp://127.0.0.1");
    }

    #[test]
    fn verify_chroot_socket_partial_match() {
        let x = chroot_socket_path("ipc:///run_snail/snail.sock", "/run");
        assert!(x.is_err());
    }
}
