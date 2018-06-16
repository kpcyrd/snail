// #![warn(unused_extern_crates)]
extern crate hlua_badtouch as hlua;
#[macro_use] extern crate structopt;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate trust_dns_resolver;
// extern crate reqwest;
extern crate kuchiki;
extern crate regex;
extern crate nix;
extern crate zmq;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

extern crate hyper;
extern crate http;
extern crate hyper_rustls;
extern crate rustls;
extern crate tokio_core;
extern crate futures;
extern crate ct_logs;
extern crate webpki_roots;

pub use failure::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub mod args;
pub mod config;
pub mod decap;
pub mod dhcp;
pub mod dns;
pub mod ipc;
pub mod scripts;
pub mod utils;
pub mod web;
pub mod wifi;
