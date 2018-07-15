#![recursion_limit = "128"]

#![warn(unused_extern_crates)]
extern crate hlua_badtouch as hlua;
#[macro_use] extern crate structopt;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate kuchiki;
extern crate regex;
extern crate nix;
extern crate zmq;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate rand;
extern crate base64;
extern crate serde_urlencoded;
extern crate toml;
extern crate users;
extern crate syscallz;
extern crate caps;
extern crate url;

extern crate trust_dns_resolver;
extern crate trust_dns_server;
extern crate trust_dns_proto;
extern crate tokio_udp;
extern crate tokio;
extern crate mrsc;

extern crate hyper;
extern crate http;
extern crate hyper_rustls;
extern crate rustls;
extern crate tokio_core;
extern crate futures;
extern crate ct_logs;
extern crate webpki_roots;

pub mod errors {
    pub use failure::{Error, ResultExt};
    pub type Result<T> = ::std::result::Result<T, Error>;
}
pub use errors::Result;

pub mod args;
pub mod config;
pub mod connect;
pub mod decap;
pub mod dhcp;
pub mod dns;
pub mod html;
pub mod json;
pub mod ipc;
pub mod recursor;
pub mod runtime;
pub mod sandbox;
pub mod scripts;
pub mod structs;
pub mod utils;
pub mod web;
pub mod wifi;
