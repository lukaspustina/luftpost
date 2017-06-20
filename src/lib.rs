#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate hyper;
extern crate lettre;
extern crate tokio_core;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tabwriter;
extern crate toml;

pub mod config;
pub use config::Config;

pub mod measurements;
pub mod output;
pub mod sensors;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
