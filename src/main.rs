mod chrome;
mod config;
mod kdbx;
mod keyring;
mod pass;
mod pwsafe;
mod utils;

use clap::{App, Arg};
use config::{parse_config, ProfileTypes};
use utils::check_dependencies;

fn main() {
    let matches = App::new("PassUp")
        .version("0.1")
        .author("Gabriel V. <gabriel.vukovic@student.tugraz.com>")
        .about("Automatically updates password databases of keepass, pass, PasswordSafe and Chrome")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Where <FILE> points to the toml configuration file")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap_or("config.toml");

    let config = match parse_config(config_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Configuration Error: {}", err);
            return;
        }
    };

    if let Err(err) = check_dependencies(&config) {
        eprintln!("Dependency Error: {}", err);
        return;
    };

    if config.profile_.type_.eq(&ProfileTypes::Kdbx) {
        kdbx::run(&config);
    } else if config.profile_.type_.eq(&ProfileTypes::Pass) {
        pass::run(&config);
    } else if config.profile_.type_.eq(&ProfileTypes::Pwsafe) {
        pwsafe::run(&config);
    } else if config.profile_.type_.eq(&ProfileTypes::ChromeG)
        || config.profile_.type_.eq(&ProfileTypes::ChromeK)
    {
        chrome::run(&config);
    }
}
