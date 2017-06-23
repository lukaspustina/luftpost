extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate luftpost;
extern crate tokio_core;

use clap::{Arg, App, Shell};
use futures::future::join_all;
use luftpost::{Config, Measurement, Sensor};
use std::io;
use std::path::Path;
use tokio_core::reactor::Core;

static BIN_NAME: &'static str = "luftpost";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

error_chain! {
    errors {
    }
    links {
        ConfigError(luftpost::config::Error, luftpost::config::ErrorKind);
        ReadingMeasurementFailed(luftpost::sensor::Error, luftpost::sensor::ErrorKind);
    }
    foreign_links {
        IoError(std::io::Error);
    }
}

quick_main!(run);

fn run() -> Result<i32> {
    let cli_args = build_cli().get_matches();

    if cli_args.is_present("completions") {
        let shell = cli_args.value_of("completions").unwrap();
        build_cli().gen_completions_to(
            BIN_NAME,
            shell.parse::<Shell>().unwrap(),
            &mut io::stdout(),
        );
        return Ok(0);
    }

    let config_file = cli_args.value_of("config-file").unwrap();
    let config_path = Path::new(config_file);
    let config = Config::from_file(config_path)?;
    if cli_args.is_present("show-config") {
        println!("Config: {:?}", &config);
    }
    let print = cli_args.is_present("print");

    let mut core = Core::new()?;

    let measurements = read_measurements(&mut core, config.sensors)?;
    if print {
        println!("Measurements collected:");
        luftpost::print_measurements(&measurements);
    }
    let threshold_violations = measurements.into_iter()
        .filter(|m| luftpost::check_thresholds(&m).len() > 0)
        .collect::<Vec<_>>();
    if print {
        println!("Measurements exceeding thresholds:");
        luftpost::print_measurements(&threshold_violations);
    }


    Ok(0)
}

fn build_cli() -> App<'static, 'static> {
    App::new("luftpost")
        .version(VERSION)
        .about("Reads luftdaten.info particle matter sensors and alarms by email if measurements exceed thresholds.")
        .arg(Arg::with_name("config-file")
             .required(true)
             .short("c")
             .long("config-file")
             .value_name("FILE")
             .help("Sets the config file")
             .takes_value(true))
        .arg(Arg::with_name("print")
             .long("print")
             .help("Print results"))
        .arg(Arg::with_name("show-config")
             .long("show-config")
             .help("Prints config"))
        .arg(Arg::with_name("completions")
             .long("completions")
             .takes_value(true)
             .hidden(true)
             .possible_values(&["bash", "fish", "zsh"])
             .help("The shell to generate the script for"))
}

fn read_measurements(core: &mut Core, sensors: Vec<Sensor>) -> Result<Vec<Measurement>> {
    let client = luftpost::create_sensor_reader(core);
    let work = sensors.into_iter().map(|s| {
        let uri = s.uri.parse().unwrap();
        let response = client.get(uri);
        s.read_measurement(response)
    });

    let big_f = join_all(work);
    core.run(big_f).map_err(|e| e.into())
}
