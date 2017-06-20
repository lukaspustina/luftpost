extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate luftpost;
extern crate tokio_core;

use clap::{Arg, App, Shell};
use futures::future::join_all;
use luftpost::Config;
use luftpost::config::Sensor;
use luftpost::measurements::Measurement;
use luftpost::output::output;
use luftpost::sensors::{create_client, read_measurement};
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
        ReadingMeasurementFailed(luftpost::sensors::Error, luftpost::sensors::ErrorKind);
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
    let print_only = cli_args.is_present("print-only");

    let mut core = Core::new()?;

    let res = read_measurements(&mut core, &config.sensors)?;
    output(&res);

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
        .arg(Arg::with_name("print-only")
             .long("print-only")
             .help("Print measurements without sending e-mails only"))
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

fn read_measurements(core: &mut Core, sensors: &[Sensor]) -> Result<Vec<Measurement>> {
    let client = create_client(core);
    let work = sensors.iter().map(|s| {
        let uri = s.uri.parse().unwrap();
        let response = client.get(uri);
        read_measurement(s.name.clone(), response)
    });

    let big_f = join_all(work);
    core.run(big_f).map_err(|e| e.into())
}
