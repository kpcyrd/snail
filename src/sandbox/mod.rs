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
    seccomp::decap_stage1()?;
    info!("decap_stage 1/2 enabled");
    Ok(())
}

pub fn decap_stage2() -> Result<()> {
    chroot("/var/empty")?;
    seccomp::decap_stage2()?;
    info!("decap_stage 2/2 enabled");
    Ok(())
}

pub fn zmq_stage1() -> Result<()> {
    seccomp::zmq_stage1()?;
    info!("zmq_stage 1/2 enabled");
    Ok(())
}

pub fn zmq_stage2() -> Result<()> {
    chroot("/var/empty")?;
    seccomp::zmq_stage2()?;
    info!("zmq_stage 2/2 enabled");
    Ok(())
}
