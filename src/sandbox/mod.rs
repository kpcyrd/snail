use errors::Result;
use config::Config;

use caps::{self, CapSet, Capability};
use nix;
use nix::unistd::{Uid, Gid, setuid, setgid, setgroups};
use users;

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

pub fn can_chroot() -> Result<bool> {
    let perm_chroot = match caps::has_cap(None, CapSet::Permitted, Capability::CAP_SYS_CHROOT) {
        Ok(perm_chroot) => perm_chroot,
        Err(_) => bail!("could not check for capability"),
    };
    debug!("caps: can chroot: {:?}", perm_chroot);
    Ok(perm_chroot)
}

pub fn try_chroot(config: &Config, path: &str) -> Result<()> {
    if can_chroot()? {
        chroot(path)?;
    } else if config.security.strict_chroot {
        bail!("strict-chroot is set and process didn't chroot")
    } else {
        warn!("chroot is not enabled");
    }

    Ok(())
}

pub fn resolve_uid(config: &Config) -> Result<Option<(u32, u32)>> {
    Ok(match config.security.user {
        Some(ref user) => {
            let user = match users::get_user_by_name(&user) {
                Some(user) => user,
                None => bail!("invalid user"),
            };
            Some((user.uid(), user.primary_group_id()))
        },
        _ => None,
    })
}

pub fn drop_user(user: Option<(u32, u32)>) -> Result<()> {
    let uid = Uid::current();
    let is_root = uid.is_root();

    if is_root {
        match user {
            Some((uid, gid)) => {
                info!("setting uid to {:?}", uid);
                setgroups(&[])?;
                setgid(Gid::from_raw(gid))?;
                setuid(Uid::from_raw(uid))?;
            },
            None => {
                warn!("executing as root!");
            },
        }
    } else {
        warn!("can't drop privileges, executing as uid={}", uid);
    }

    Ok(())
}

pub fn decap_stage1() -> Result<()> {
    seccomp::decap_stage1()?;
    info!("decap_stage 1/2 enabled");
    Ok(())
}

pub fn decap_stage2(config: &Config) -> Result<()> {
    try_chroot(config, "/run/snail")?;
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

pub fn dns_stage1() -> Result<()> {
    seccomp::dns_stage1()?;
    info!("dns_stage 1/3 enabled");
    Ok(())
}

pub fn dns_stage2(config: &Config) -> Result<()> {
    let user = resolve_uid(&config)?;

    try_chroot(config, "/run/snail")?;

    drop_user(user)?;

    info!("dns_stage 2/3 enabled");
    Ok(())
}

pub fn dns_stage3() -> Result<()> {
    seccomp::dns_stage3()?;
    info!("dns_stage 3/3 enabled");
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
