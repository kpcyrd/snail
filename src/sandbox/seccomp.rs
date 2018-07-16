use syscallz::{Context, Syscall};

use errors::Result;


pub fn decap_stage1() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    ctx.allow_syscall(Syscall::nanosleep)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    #[cfg(not(target_arch="arm"))]
    ctx.allow_syscall(Syscall::mmap)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::mmap2)?;
    ctx.allow_syscall(Syscall::mprotect)?;
    ctx.allow_syscall(Syscall::clone)?;
    ctx.allow_syscall(Syscall::set_robust_list)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::prctl)?;
    ctx.allow_syscall(Syscall::munmap)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::openat)?; // TODO: remove this, needed for /etc/hosts and some /proc stuff close to dns
    ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::ioctl)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::epoll_pwait)?; // needed for stage1
    ctx.allow_syscall(Syscall::sendto)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::send)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::recv)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::fcntl64)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::madvise)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::lseek)?;
    ctx.allow_syscall(Syscall::brk)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;
    ctx.allow_syscall(Syscall::gettimeofday)?;
    ctx.allow_syscall(Syscall::stat)?; // needed for stage1
    ctx.allow_syscall(Syscall::fstat)?; // needed for stage1
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::fstat64)?; // needed for stage1
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::getdents)?; // needed for stage1
    ctx.allow_syscall(Syscall::getdents64)?; // needed for stage1
    ctx.allow_syscall(Syscall::eventfd2)?; // needed for stage1
    ctx.allow_syscall(Syscall::rt_sigprocmask)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getparam)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_setscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::getpeername)?; // needed for stage1
    ctx.allow_syscall(Syscall::chroot)?; // needed for stage1
    ctx.allow_syscall(Syscall::capget)?; // needed for stage1
    ctx.allow_syscall(Syscall::chdir)?; // needed for stage1
    ctx.allow_syscall(Syscall::getuid)?; // needed for stage1
    ctx.allow_syscall(Syscall::setgroups)?; // needed for stage1
    ctx.allow_syscall(Syscall::setgid)?; // needed for stage1
    ctx.allow_syscall(Syscall::setuid)?; // needed for stage1
    ctx.allow_syscall(Syscall::seccomp)?; // needed for stage1

    ctx.load()?;

    info!("decap_stage 1/3 is active");
    Ok(())
}

pub fn decap_stage3() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    ctx.allow_syscall(Syscall::nanosleep)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    #[cfg(not(target_arch="arm"))]
    ctx.allow_syscall(Syscall::mmap)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::mmap2)?;
    ctx.allow_syscall(Syscall::mprotect)?;
    ctx.allow_syscall(Syscall::clone)?;
    ctx.allow_syscall(Syscall::set_robust_list)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::prctl)?;
    ctx.allow_syscall(Syscall::munmap)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::openat)?; // TODO: remove this, needed for /etc/hosts and some /proc stuff close to dns
    ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::ioctl)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::epoll_pwait)?; // needed for stage1
    ctx.allow_syscall(Syscall::sendto)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::send)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::recv)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::fcntl64)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::madvise)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::lseek)?;
    ctx.allow_syscall(Syscall::brk)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;
    ctx.allow_syscall(Syscall::gettimeofday)?;

    ctx.load()?;

    info!("decap_stage 3/3 is active");
    Ok(())
}

pub fn zmq_stage1() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    ctx.allow_syscall(Syscall::brk)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;
    ctx.allow_syscall(Syscall::gettimeofday)?;
    ctx.allow_syscall(Syscall::restart_syscall)?;
    ctx.allow_syscall(Syscall::sigaltstack)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::accept4)?;
    ctx.allow_syscall(Syscall::getpeername)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::recv)?;
    ctx.allow_syscall(Syscall::sendto)?;
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::send)?;
    ctx.allow_syscall(Syscall::openat)?; // needed for stage1
    ctx.allow_syscall(Syscall::stat)?; // needed for stage1
    ctx.allow_syscall(Syscall::fstat)?; // needed for stage1
    #[cfg(target_arch = "arm")]
    ctx.allow_syscall(Syscall::fstat64)?; // needed for stage1
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::getdents)?; // needed for stage1
    ctx.allow_syscall(Syscall::getdents64)?; // needed for stage1
    ctx.allow_syscall(Syscall::close)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getaffinity)?; // needed for stage1
    #[cfg(not(target_arch="arm"))]
    ctx.allow_syscall(Syscall::mmap)?; // needed for stage1
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::mmap2)?; // needed for stage1
    ctx.allow_syscall(Syscall::madvise)?; // needed for stage1
    ctx.allow_syscall(Syscall::mprotect)?; // needed for stage1
    ctx.allow_syscall(Syscall::munmap)?; // needed for stage1
    ctx.allow_syscall(Syscall::eventfd2)?; // needed for stage1
    ctx.allow_syscall(Syscall::fcntl)?; // needed for stage1
    #[cfg(target_arch="arm")]
    ctx.allow_syscall(Syscall::fcntl64)?; // needed for stage1
    ctx.allow_syscall(Syscall::getrandom)?; // needed for stage1
    ctx.allow_syscall(Syscall::epoll_create1)?; // needed for stage1
    ctx.allow_syscall(Syscall::epoll_ctl)?; // needed for stage1
    ctx.allow_syscall(Syscall::clone)?; // needed for stage1
    ctx.allow_syscall(Syscall::rt_sigprocmask)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getparam)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_setscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::set_robust_list)?; // needed for stage1
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::unlink)?; // needed for stage1
    ctx.allow_syscall(Syscall::unlinkat)?; // needed for stage1
    ctx.allow_syscall(Syscall::socket)?; // needed for stage1
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::epoll_wait)?; // needed for stage1
    ctx.allow_syscall(Syscall::epoll_pwait)?; // needed for stage1
    ctx.allow_syscall(Syscall::bind)?; // needed for stage1
    ctx.allow_syscall(Syscall::listen)?; // needed for stage1
    ctx.allow_syscall(Syscall::getsockname)?; // needed for stage1
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::chmod)?; // needed for stage1
    ctx.allow_syscall(Syscall::fchmodat)?; // needed for stage1
    ctx.allow_syscall(Syscall::connect)?; // needed for stage1
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::chown)?; // needed for stage1
    ctx.allow_syscall(Syscall::chroot)?; // needed for stage1
    ctx.allow_syscall(Syscall::chdir)?; // needed for stage1
    ctx.allow_syscall(Syscall::prctl)?; // needed for stage1
    ctx.allow_syscall(Syscall::seccomp)?; // needed for stage1

    ctx.allow_syscall(Syscall::gettid)?; // needed for stage1
    ctx.allow_syscall(Syscall::tgkill)?; // needed for stage1

    ctx.load()?;

    info!("zmq_stage 1/3 is active");
    Ok(())
}

pub fn zmq_stage3() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    #[cfg(not(target_arch = "aarch64"))]
    ctx.allow_syscall(Syscall::poll)?;
    #[cfg(target_arch = "aarch64")]
    ctx.allow_syscall(Syscall::ppoll)?;
    ctx.allow_syscall(Syscall::brk)?;
    ctx.allow_syscall(Syscall::clock_gettime)?;
    ctx.allow_syscall(Syscall::gettimeofday)?;
    ctx.allow_syscall(Syscall::restart_syscall)?;

    ctx.load()?;

    info!("zmq_stage 3/3 is active");
    Ok(())
}

pub fn dns_stage1() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::getsockname)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::ioctl)?;
    ctx.allow_syscall(Syscall::mmap)?; // needed for stage1
    ctx.allow_syscall(Syscall::mprotect)?; // needed for stage1
    ctx.allow_syscall(Syscall::clone)?; // needed for stage1
    ctx.allow_syscall(Syscall::openat)?; // needed for stage1
    ctx.allow_syscall(Syscall::set_robust_list)?; // needed for stage1
    ctx.allow_syscall(Syscall::sigaltstack)?; // needed for stage1
    ctx.allow_syscall(Syscall::fstat)?; // needed for stage1
    ctx.allow_syscall(Syscall::munmap)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getaffinity)?; // needed for stage1
    ctx.allow_syscall(Syscall::capget)?; // needed for stage1
    ctx.allow_syscall(Syscall::getuid)?; // needed for stage1
    ctx.allow_syscall(Syscall::chroot)?; // needed for stage1
    ctx.allow_syscall(Syscall::chdir)?; // needed for stage1
    ctx.allow_syscall(Syscall::getpid)?; // needed for stage1
    ctx.allow_syscall(Syscall::tgkill)?; // needed for stage1
    ctx.allow_syscall(Syscall::setgroups)?; // needed for stage1
    ctx.allow_syscall(Syscall::setgid)?; // needed for stage1
    ctx.allow_syscall(Syscall::setuid)?; // needed for stage1
    ctx.allow_syscall(Syscall::rt_sigreturn)?; // needed for stage1
    ctx.allow_syscall(Syscall::madvise)?; // needed for stage1
    ctx.allow_syscall(Syscall::exit_group)?; // needed for stage1
    ctx.allow_syscall(Syscall::prctl)?; // needed for stage1
    ctx.allow_syscall(Syscall::seccomp)?; // needed for stage1

    ctx.load()?;

    info!("dns_stage 1/3 is active");
    Ok(())
}

pub fn dns_stage3() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::epoll_ctl)?;
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::getrandom)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    ctx.allow_syscall(Syscall::socket)?;
    ctx.allow_syscall(Syscall::pipe2)?;
    ctx.allow_syscall(Syscall::epoll_create1)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::getsockname)?;
    ctx.allow_syscall(Syscall::bind)?;
    ctx.allow_syscall(Syscall::ioctl)?;

    ctx.load()?;

    info!("dns_stage 3/3 is active");
    Ok(())
}
