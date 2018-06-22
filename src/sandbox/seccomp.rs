use syscallz::{Context, Syscall};

use errors::Result;


pub fn decap_stage1() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    ctx.allow_syscall(Syscall::poll)?;
    ctx.allow_syscall(Syscall::nanosleep)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::mmap)?;
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
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::madvise)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::lseek)?;
    ctx.allow_syscall(Syscall::fstat)?; // needed for stage1
    ctx.allow_syscall(Syscall::getdents)?; // needed for stage1
    ctx.allow_syscall(Syscall::eventfd2)?; // needed for stage1
    ctx.allow_syscall(Syscall::rt_sigprocmask)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getparam)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_setscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::getpeername)?; // needed for stage1
    ctx.allow_syscall(Syscall::chroot)?; // needed for stage1
    ctx.allow_syscall(Syscall::chdir)?; // needed for stage1
    ctx.allow_syscall(Syscall::seccomp)?; // needed for stage1

    ctx.load()?;

    info!("decap_stage 1/2 is active");
    Ok(())
}

pub fn decap_stage2() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    ctx.allow_syscall(Syscall::poll)?;
    ctx.allow_syscall(Syscall::nanosleep)?;
    ctx.allow_syscall(Syscall::sched_getaffinity)?;
    ctx.allow_syscall(Syscall::mmap)?;
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
    ctx.allow_syscall(Syscall::epoll_wait)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::close)?;
    ctx.allow_syscall(Syscall::fcntl)?;
    ctx.allow_syscall(Syscall::connect)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::setsockopt)?;
    ctx.allow_syscall(Syscall::exit_group)?;
    ctx.allow_syscall(Syscall::madvise)?;
    ctx.allow_syscall(Syscall::exit)?;
    ctx.allow_syscall(Syscall::lseek)?;

    ctx.load()?;

    info!("decap_stage 2/2 is active");
    Ok(())
}

pub fn zmq_stage1() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    ctx.allow_syscall(Syscall::poll)?;
    ctx.allow_syscall(Syscall::accept4)?;
    ctx.allow_syscall(Syscall::getpeername)?;
    ctx.allow_syscall(Syscall::getsockopt)?;
    ctx.allow_syscall(Syscall::recvfrom)?;
    ctx.allow_syscall(Syscall::sendto)?;
    ctx.allow_syscall(Syscall::openat)?; // needed for stage1
    ctx.allow_syscall(Syscall::fstat)?; // needed for stage1
    ctx.allow_syscall(Syscall::getdents)?; // needed for stage1
    ctx.allow_syscall(Syscall::close)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getaffinity)?; // needed for stage1
    ctx.allow_syscall(Syscall::mmap)?; // needed for stage1
    ctx.allow_syscall(Syscall::mprotect)?; // needed for stage1
    ctx.allow_syscall(Syscall::munmap)?; // needed for stage1
    ctx.allow_syscall(Syscall::eventfd2)?; // needed for stage1
    ctx.allow_syscall(Syscall::fcntl)?; // needed for stage1
    ctx.allow_syscall(Syscall::getrandom)?; // needed for stage1
    ctx.allow_syscall(Syscall::epoll_create1)?; // needed for stage1
    ctx.allow_syscall(Syscall::epoll_ctl)?; // needed for stage1
    ctx.allow_syscall(Syscall::clone)?; // needed for stage1
    ctx.allow_syscall(Syscall::rt_sigprocmask)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getparam)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_getscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::sched_setscheduler)?; // needed for stage1
    ctx.allow_syscall(Syscall::set_robust_list)?; // needed for stage1
    ctx.allow_syscall(Syscall::unlink)?; // needed for stage1
    ctx.allow_syscall(Syscall::socket)?; // needed for stage1
    ctx.allow_syscall(Syscall::epoll_wait)?; // needed for stage1
    ctx.allow_syscall(Syscall::bind)?; // needed for stage1
    ctx.allow_syscall(Syscall::listen)?; // needed for stage1
    ctx.allow_syscall(Syscall::getsockname)?; // needed for stage1
    ctx.allow_syscall(Syscall::chmod)?; // needed for stage1
    ctx.allow_syscall(Syscall::connect)?; // needed for stage1
    ctx.allow_syscall(Syscall::chown)?; // needed for stage1
    ctx.allow_syscall(Syscall::chroot)?; // needed for stage1
    ctx.allow_syscall(Syscall::chdir)?; // needed for stage1
    ctx.allow_syscall(Syscall::prctl)?; // needed for stage1
    ctx.allow_syscall(Syscall::seccomp)?; // needed for stage1

    ctx.allow_syscall(Syscall::gettid)?; // needed for stage1
    ctx.allow_syscall(Syscall::tgkill)?; // needed for stage1

    ctx.load()?;

    info!("zmq_stage 1/2 is active");
    Ok(())
}

pub fn zmq_stage2() -> Result<()> {
    let mut ctx = Context::init()?;

    ctx.allow_syscall(Syscall::futex)?;
    ctx.allow_syscall(Syscall::read)?;
    ctx.allow_syscall(Syscall::write)?;
    ctx.allow_syscall(Syscall::getpid)?;
    ctx.allow_syscall(Syscall::poll)?;

    ctx.load()?;

    info!("zmq_stage 2/2 is active");
    Ok(())
}
