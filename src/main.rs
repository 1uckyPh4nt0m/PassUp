use std::env;

mod pass;
mod kdbx;
mod utils;

extern crate clap;
use clap::{Arg, App};
extern crate toml;
use toml::Value;

use std::fs;

fn main() {
    let matches = App::new("PassUp")
                          .version("0.1")
                          .author("Gabriel V. <gabriel.vuk@gmail.com>")
                          .about("Automatically updates password databases of pass and keepass")
                          .arg(Arg::with_name("config")
                                .short("c")
                                .long("config")
                                .value_name("FILE")
                                .help("Sets a custom config file")
                                .takes_value(true)
                                .required(true))
                          .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config_path = matches.value_of("config").unwrap_or("config.toml");
    println!("Path of config: {}", config_path);

    let config_r = fs::read(config_path);
    let config = match config_r {
        Ok(config) => String::from_utf8(config).unwrap(),
        Err(e) => panic!("Reading the config file: {}", e),
    };
    let value = config.parse::<Value>().unwrap();

    let db_type = match value["type_of_db"].as_str() {
        Some(db_type) => db_type,
        None => panic!("type_of_db missing in provided config file!"),
    };
    if db_type.eq("kdbx") {
        let path = match value["path_to_kdbx"].as_str() {
            Some(path) => path,
            None => panic!("path_to_kdbx missing in provided config file!"),
        };
        let db_password = match value["kdbx_password"].as_str() {
            Some(db_password) => db_password,
            None => panic!("kdbx_password missing in provided config file!"),
        };
        kdbx::run(path, db_password);
    } else if db_type.eq("pass") {
        pass::run();
    } else {
        print_usage();
    }
}

fn print_usage() {
    let args: Vec<String> = env::args().collect();

    println!("Usage:");
    println!("      {} --config=path_to_config", args[0]);
    println!("      {} -c path_to_config", args[0]);
}