extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate lettre;
extern crate luftpost;
extern crate tokio_core;

use clap::{Arg, App, Shell};
use futures::future::join_all;
use luftpost::{Config, Mailer, Measurement, Sensor};
use luftpost::config::NotificationCondition;
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
        EmailError(luftpost::mail::Error, luftpost::mail::ErrorKind);
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
    if cli_args.is_present("check-config") {
        println!("Configuration is ok");
        return Ok(0);
    }
    let print = cli_args.is_present("print");

    let mut core = Core::new()?;

    let measurements = read_measurements(&mut core, config.sensors)?;
    if print {
        println!("Measurements collected:");
        luftpost::print_measurements(measurements.iter().map(|m| m).collect::<Vec<_>>().as_slice());
    }
    let checked_measurements = measurements
        .into_iter()
        .map(|m| luftpost::check_measurement(m))
        .collect::<Vec<_>>();
    if print {
        println!("Measurements exceeding thresholds:");
        let violations = checked_measurements.iter().filter(|m| !m.violations.is_empty()).map(|cm| &cm.measurement).collect::<Vec<_>>();
        luftpost::print_measurements(violations.as_slice())
    }

    if let Some(ref smtp) = config.smtp {
        if print {
            println!("Sending E-Mails:");
        }
        let mut mailer = Mailer::create_mailer(smtp)?;
        let results = checked_measurements
            .iter()
            .filter(|cm|
                match cm.measurement.sensor.notification_condition.unwrap() {
                    NotificationCondition::Always | NotificationCondition::ThresholdExceeded if !cm.violations.is_empty() => true,
                    _ => false
                }
            )
            .map(|cm| {
                println!("{}", cm.measurement.sensor.name);
                mailer.mail_measurement(&cm.measurement).map_err(|e| e.into())
            });
        results.collect::<::std::result::Result<Vec<()>, Error>>()?;
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
        .arg(Arg::with_name("check-config")
             .long("check-config")
             .help("Checks config and exits"))
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
        let uri = s.data_uri.parse().unwrap();
        let response = client.get(uri);
        s.read_measurement(response)
    });

    let big_f = join_all(work);
    core.run(big_f).map_err(|e| e.into())
}
