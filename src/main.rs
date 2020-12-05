mod pass;
mod kdbx;
mod utils;

extern crate clap;
use clap::{Arg, App};
use utils::parse_config;


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
    println!("Path of config: {}", config_path);

    let config = parse_config(config_path);

    if config.type_of_db_.eq("kdbx") {
        kdbx::run(config.path_to_kdbx_, config.kdbx_password_, config.script_dir_, config.blacklist_);
    } else if config.type_of_db_.eq("pass") {
        pass::run(config.script_dir_, config.blacklist_);
    }
}