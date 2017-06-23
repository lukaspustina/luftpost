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

pub mod check;
pub mod config;
pub mod measurement;
pub mod output;
pub mod sensor;

pub use check::check_thresholds;
pub use config::Config;
pub use measurement::Measurement;
pub use sensor::Sensor;
pub use sensor::create_sensor_reader;
pub use output::print_measurements;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
