mod pass;
mod kdbx;
mod passwsafe;
mod config;
mod utils;
mod chrome;

extern crate snafu;
extern crate clap;
extern crate which;

use clap::{Arg, App};
use config::parse_config;
use utils::check_dependencies;

fn main() {

    chrome::test();

    let matches = App::new("PassUp")
                            .version("0.1")
                            .author("Gabriel V. <gabriel.vukovic@student.tugraz.com>")
                            .about("Automatically updates password databases of pass and keepass")
                            .arg(Arg::with_name("config")
                                .short("c")
                                .long("config")
                                .value_name("FILE")
                                .help("Where <FILE> points to the toml configuration file")
                                .takes_value(true)
                                .required(true))
                            .get_matches();

    let config_path = matches.value_of("config").unwrap_or("config.toml");

    let config = match parse_config(config_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Configuration Error: {}", err);
            return;
        }
    };

    match check_dependencies(&config) {
        Ok(()) => (),
        Err(err) => {
            eprintln!("Dependency Error: {}", err);
            return;
        }
    };

    if config.profile_.type_.eq("kdbx") {
        kdbx::run(&config);
    } else if config.profile_.type_.eq("pass") {
        pass::run(&config);
    } else if config.profile_.type_.eq("pwsafe") {
        passwsafe::run(&config);
    }
}