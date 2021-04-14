mod pass;
mod kdbx;
mod config;
mod utils;

extern crate snafu;
extern crate clap;
use clap::{Arg, App};
use config::{parse_config};

fn main() {
    let matches = App::new("PassUp")
                            .version("0.1")
                            .author("Gabriel V. <gabriel.vukovic@student.tugraz.com>")
                            .about("Automatically updates password databases of pass and keepass")
                            .arg(Arg::with_name("config")
                                .short("c")
                                .long("config")
                                .value_name("FILE")
                                .help("Sets a custom config file")
                                .takes_value(true)
                                .required(true))
                            .get_matches();

    let config_path = matches.value_of("config").unwrap_or("config.toml");

    let config = match parse_config(config_path) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    if config.profile_.type_.eq("kdbx") {
        kdbx::run(&config);
    } else if config.profile_.type_.eq("pass") {
        pass::run(&config);
    }
}