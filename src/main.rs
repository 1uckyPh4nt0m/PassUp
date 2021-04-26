mod pass;
mod kdbx;
mod config;
mod utils;

extern crate snafu;
extern crate clap;
extern crate which;

use clap::{Arg, App};
use config::{parse_config};
use which::which;

//TODO: Errors für pass und utils anlegen
//TODO: Updateprozess beschleunigen via Multithreading
//TODO: [urls] funktionalität hinzufügen
//TODO: Beispiel Nightwatch scripts erstellen

fn main() {
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
            eprintln!("Error: {}", err);
            return;
        }
    };

    //TODO maybe allow to specify path to nightwatch
    //TODO move to utils and create errors
    match which("nightwatch") {
        Ok(_) => (),
        Err(_) => {
            eprintln!("The binary nightwatch was not found! Please install Nightwatch, refer to the README.md for help.");
            return;
        }
    }
    if config.browser_type_.eq("firefox") {
        match which("firefox") {
            Ok(_) => (),
            Err(_) => {
                eprintln!("The binary firefox was not found! Please install Firefox, refer to the README.md for help.");
                return;
            }
        }
    } else if config.browser_type_.eq("chrome") {
        match which("google-chrome") {
            Ok(_) => (),
            Err(_) => {
                eprintln!("The binary google-chrome was not found! Please install Chrome, refer to the README.md for help.");
                return;
            }
        }
    } else {
        eprintln!("Browser type is neither 'firefox' nor 'chrome'! This should not happen!");
        return;
    }

    if config.profile_.type_.eq("kdbx") {
        kdbx::run(&config);
    } else if config.profile_.type_.eq("pass") {
        pass::run(&config);
    }
}