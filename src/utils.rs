use std::process::{Command, Output};

extern crate passwords;
extern crate toml;
use passwords::PasswordGenerator;
use toml::Value;
use std::fs;

pub struct Configuration {
    pub type_of_db_: String,
    pub script_dir_: String,
    pub path_to_kdbx_: String,
    pub kdbx_password_: String,
    pub blacklist_: Vec<String>,
}

pub fn get_pw() -> String {
    let pg = PasswordGenerator {
        length: 15,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: false,
        strict: true,
        exclude_similar_characters: true,
        spaces: false,
    };

    return pg.generate_one().unwrap();
}

pub fn cmd(program: &str, args: &[&str]) -> Output {
    return Command::new(program)
        .args(args)
        .output()
        .expect("failed to execute process");
}

pub fn exec_nightwatch(script_path: &str, url: &str, username: &str, old_pass: &str, new_pass: &str) -> Output {
    let output = cmd("nightwatch", 
            &["--headless", "--env", "firefox", "--test", script_path, 
            url, username, old_pass, new_pass]);
    return output;
}

pub fn parse_config(path: &str) -> Configuration {
    let config_r = fs::read(path);
    let config = match config_r {
        Ok(config) => String::from_utf8(config).unwrap_or(String::new()),
        Err(e) => panic!("Reading the config file: {}", e),
    };

    let value_r = config.parse::<Value>();
    let config_map = match value_r {
        Ok(config_map) => config_map,
        Err(e) => panic!("parsing config file: {}", e),
    };

    let db_type = match config_map["type_of_db"].as_str() {
        Some(db_type) => db_type.to_string(),
        None => panic!("type_of_db missing in provided config file!"),
    };

    let script_dir = match config_map["script_dir"].as_str() {
        Some(script_dir) => script_dir.to_string(),
        None => panic!("script_dir missing in provided config file!"),
    };

    let path_to_kdbx = match config_map["path_to_kdbx"].as_str() {
        Some(path) => path.to_string(),
        None => String::new(),
    };

    let kdbx_password = match config_map["kdbx_password"].as_str() {
        Some(db_password) => db_password.to_string(),
        None => String::new(),
    };

    let blacklist_v: Vec<Value> = match config_map["blacklist"].as_array() {
        Some(db_password) => db_password.to_vec(),
        None => Vec::new(),
    };

    let blacklist = blacklist_v.iter().map(|domain| domain.as_str().unwrap_or("").to_string()).collect();

    let configuration = Configuration{ type_of_db_: db_type, script_dir_: script_dir, path_to_kdbx_: path_to_kdbx, kdbx_password_: kdbx_password, blacklist_: blacklist};
    return configuration;
}