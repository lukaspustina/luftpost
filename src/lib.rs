#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate handlebars;
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
pub mod mail;
pub mod measurement;
pub mod output;
pub mod sensor;

pub use check::check_measurement;
pub use config::Config;
pub use mail::Mailer;
pub use measurement::Measurement;
pub use sensor::{Sensor, create_sensor_reader};
pub use output::print_measurements;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;
