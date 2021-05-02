extern crate passwords;
extern crate toml;
extern crate url;

use std::{process::{Command, Output}, sync::mpsc::Sender};
use kpdb::EntryUuid;
use passwords::PasswordGenerator;
use threadpool::ThreadPool;
use std::path::PathBuf;
use url::{Url};
use crate::config::{Configuration, Script, Source};
use std::io;

#[derive(Clone)]
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

pub struct ThreadResult {
    pub db_entry_: DBEntry,
    pub result_: Result<Output, std::io::Error>
}

impl ThreadResult {
    fn new(db_entry_: DBEntry, result_: Result<Output, std::io::Error>) -> Self { Self { db_entry_, result_ } }
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

pub fn cmd(program: &str, args: &[&str], port: &str) -> io::Result<Output> {
    return Command::new(program)
        .args(args)
        .env("PORT", port)
        .output();
}

pub fn exec_nightwatch(script_path: &str, db_entry: &DBEntry, browser_type: &String, port: &String) -> io::Result<Output> {
    let output = cmd("nightwatch", 
            &["--env", browser_type, "--test", script_path,
            &db_entry.url_, &db_entry.username_, &db_entry.old_password_, &db_entry.new_password_], port);
    
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

pub fn get_script_path(scripts: &Vec<Script>, blocklist: &Vec<String>, db_entry: &DBEntry) -> Option<String> {
    for script in scripts {
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

        if !script_path.exists() {
            // eprintln!("Script {} not present!", script_path.to_str().unwrap());
            // return None;
            continue;
        }

        let script_path_string = match script_path.to_str() {
            Some(path) => path.to_owned(),
            None => return None
        };

        if script.blocklist_.contains(&script_path_string) {
            return None;
        }

        return Some(script_path_string);
    }
    return None;
}

pub fn check_port_available(port: u16) -> bool {
    match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn run_update_threads(db: &DB, blocklist: &Vec<String>, config: &Configuration, tx: Sender<ThreadResult>) -> usize {
    let mut port = 4444u16;
    let mut nr_jobs = 0usize;
    let pool = ThreadPool::new(config.nr_threads_);
    for db_entry in db.entries.iter() {
        let entry = db_entry.clone();
        let script_path = match get_script_path(&config.scripts_, blocklist, &db_entry) {
            Some(path) => path,
            None => continue
        };
        
        let browser_type = config.browser_type_.to_owned();
        
        nr_jobs += 1;
        let tx = tx.clone();
        pool.execute(move || {
            match exec_nightwatch(&script_path, &entry, &browser_type, &port.to_string()) {
                Ok(output) => tx.send(ThreadResult::new(entry, Ok(output))).expect("Error: Thread could not send"),
                Err(err) => tx.send(ThreadResult::new(entry, Err(err))).expect("Error: Thread could not send")
            };
        });
        // TODO check if port available
        //while !check_port_available(port) {
        port += 1;
        //}
    }
    return nr_jobs;
}