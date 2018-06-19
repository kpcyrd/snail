use structopt::StructOpt;
use structopt::clap::Shell;
use std::io;

pub mod snailctl;
pub mod snaild;

#[inline]
pub fn gen_completions<T: StructOpt>(bin_name: &str) {
    T::clap()
        .gen_completions_to(bin_name, Shell::Bash, &mut io::stdout());
}
