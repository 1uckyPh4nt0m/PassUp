extern crate passwords;
extern crate toml;
extern crate url;

use std::process::{Command, Output};
use kpdb::EntryUuid;
use passwords::PasswordGenerator;
use std::path::PathBuf;
use url::{Url};
use crate::config::{Script};

pub struct DBEntry {
    pub url_: String,
    pub username_: String,
    pub old_password_: String,
    pub new_password_: String,
    pub uuid: EntryUuid
}

impl DBEntry {
    pub fn new(url_: String, username_: String, old_password_: String, new_password_: String, uuid: EntryUuid) -> Self { Self { url_, username_, old_password_, new_password_, uuid } }
}

pub struct DB {
    pub entries: Vec<DBEntry>
}

impl DB {
    pub fn new(entries: Vec<DBEntry>) -> Self { Self { entries } }
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

pub fn cmd(program: &str, args: &[&str]) -> Option<Output> {
    return Command::new(program)
        .args(args)
        .output().ok();
}

pub fn exec_nightwatch(script_path: &str, db_entry: &DBEntry) -> Option<Output> {
    let output = cmd("nightwatch", 
            &["--env", "firefox", "--test", script_path, 
            &db_entry.url_, &db_entry.username_, &db_entry.old_password_, &db_entry.new_password_]);
    
    return output;
}

pub fn get_script_name_check_blocklist(url: &String, blocklist: &Vec<String>) -> Option<String> {
    let mut url_;
    if !url.contains("https://") {
        url_ = "https://".to_owned();
        url_.push_str(&url);
    } else {
        url_ = url.to_owned();
    }
    
    let target_url = match Url::parse(&url_) {
        Ok(target_url) => target_url,
        Err(_) => {
            eprintln!("Could not parse URL {}", url_);
            return None;
        }
    };

    let mut target_domain = match target_url.domain() {
        Some(domain) => domain.to_owned(),
        None => {
            eprintln!("URL does not contain a domain name: {}", url);
            return None;
        }
    };

    if blocklist.contains(&target_domain.to_string()) {
        return None;
    }

    target_domain.push_str(".js");

    return Some(target_domain);
}

pub fn get_script_path(script: &Script, blocklist: &Vec<String>, db_entry: &DBEntry) -> Option<String> {
    let mut script_path = PathBuf::new();
    script_path.push(&script.dir_);

    let script_name = match get_script_name_check_blocklist(&db_entry.url_, blocklist) {
        Some(target) => target,
        None => return None
    };

    script_path.push(&script_name);
    let path = script_path.to_str().unwrap_or("");
    if path.eq("") {
        eprintln!("Could not unwrap script path. Skipping site \"{}\"!", db_entry.url_);
        return None;
    }

    let entry_is_file = script_path.is_file();
    if entry_is_file == false {
        eprintln!("Script {} not present!", script_path.to_str().unwrap());
        return None;
    }

    let script_path_string = match script_path.to_str() {
        Some(path) => path.to_owned(),
        None => return None
    };

    if script.blocklist_.contains(&script_path_string) {
        return None;
    }

    return Some(script_path_string)
}

pub fn exec_script(script: &Script, blocklist: &Vec<String>, db_entry: &DBEntry) -> Option<Output> {
    let script_path = match get_script_path(script, blocklist, &db_entry) {
        Some(path) => path,
        None => return None
    };

    return exec_nightwatch(&script_path, &db_entry);
}