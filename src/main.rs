extern crate clap;
#[macro_use]
extern crate error_chain;
extern crate luftpost;

use clap::{Arg, App, Shell};
use luftpost::Config;
use std::io;
use std::path::Path;

static BIN_NAME: &'static str = "luftpost";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

error_chain! {
    errors {
    }
    links {
        ConfigError(luftpost::config::Error, luftpost::config::ErrorKind);
    }
}

quick_main!(run);

fn run() -> Result<i32> {
    let cli_args = build_cli().get_matches();

    if cli_args.is_present("completions") {
        let shell= cli_args.value_of("completions").unwrap();
        build_cli().gen_completions_to(BIN_NAME, shell.parse::<Shell>().unwrap(), &mut io::stdout());
        return Ok(0);
    }

    let config_file = cli_args.value_of("config-file").unwrap();
    let config_path = Path::new(config_file);
    let config = Config::from_file(config_path)?;
    if cli_args.is_present("show-config") {
        println!("Config: {:?}", &config);
    }
    let print_only = cli_args.is_present("print-only");

    Err("error".into())
}

fn build_cli() -> App<'static, 'static> {
    let app = App::new("luftpost")
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
             .help("The shell to generate the script for"));

    app
}

