use errors::Result;

use nix;

use std::env;

pub mod seccomp;
pub mod syscalls;


pub fn chroot(path: &str) -> Result<()> {
    nix::unistd::chroot(path)?;
    env::set_current_dir("/")?;
    Ok(())
}

pub fn decap_stage1() -> Result<()> {
    warn!("decap_stage1 is unimplemented");
    Ok(())
}

pub fn decap_stage2() -> Result<()> {
    chroot("/var/empty")?;
    warn!("decap_stage2 is unimplemented");
    Ok(())
}

pub fn zmq_stage1() -> Result<()> {
    warn!("zmq_stage1 is unimplemented");
    Ok(())
}

pub fn zmq_stage2() -> Result<()> {
    chroot("/var/empty")?;
    warn!("zmq_stage2 is unimplemented");
    Ok(())
}
