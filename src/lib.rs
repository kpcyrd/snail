extern crate hlua_badtouch as hlua;
#[macro_use] extern crate structopt;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate reqwest;
// extern crate kuchiki;

pub use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub mod args;
pub mod decap;
pub mod scripts;
pub mod utils;
