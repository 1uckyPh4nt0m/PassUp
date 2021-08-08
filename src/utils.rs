extern crate passwords;
extern crate toml;
extern crate url;

use std::{collections::HashMap, process::{Command, Output}, sync::mpsc::Sender};
use kpdb::EntryUuid;
use passwords::PasswordGenerator;
use threadpool::ThreadPool;
use std::path::PathBuf;
use url::{Url};
use crate::config::{BrowserType, Configuration};
use std::io;
use snafu::{ResultExt, Snafu};
use which::which;
use crate::utils;
use regex::Regex;

const FIREFOX_PORT: u16 = 4444;
const CHROME_PORT: u16 = 9515;

#[derive(Debug, Clone, PartialEq)]
pub enum Uuid {
    None,
    Kdbx(EntryUuid)
} 

#[derive(Debug, Clone)]
pub struct DBEntry {
    pub url_: String,
    pub username_: String,
    pub old_password_: String,
    pub new_password_: String,
    pub uuid_: Uuid
}

impl DBEntry {
    pub fn new(url_: String, username_: String, old_password_: String, new_password_: String) -> Self { Self { url_, username_, old_password_, new_password_, uuid_: Uuid::None} }
}

#[derive(Debug)]
pub struct DB {
    pub entries: Vec<DBEntry>
}

impl DB {
    pub fn new(entries: Vec<DBEntry>) -> Self { Self { entries } }
}

#[derive(Debug, Snafu)]
pub enum LibraryError {
    UrlError { source: url::ParseError },
    IoError { source: io::Error },
    RegexLibError { source: regex::Error }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A new password could not be generated: {}", err))]
    PasswordGeneratorError { err: &'static str },
    #[snafu(display("Could not execute command \'{}{}\': {}", program, args, source))]
    CmdError { program: &'static str, args: String, source: LibraryError },
    #[snafu(display("Could not parse URL \'{}\' with {}", url, source))]
    UrlParseError { url: String, source: LibraryError },
    #[snafu(display("URL does not contain a domain name: {}", url))]
    UrlDomainError { url: String },
    UrlDomainBlocked,
    #[snafu(display("Script path does not result in a valid unicode string. Skipping site: {}", url))]
    ScriptPathError { url: String },
    #[snafu(display("Script path \'{}\' is not present", path))]
    ScriptMissingError { path: String },
    ScriptBlocked,
    #[snafu(display("Warning: Script for website \'{}\' with username: \'{}\' did not execute succesfully\n{}", db_entry.url_, db_entry.username_, std::str::from_utf8(&output.stdout).unwrap_or("error")))]
    NightwatchExecError { db_entry: DBEntry, output: Output },
    #[snafu(display("The binary {} was not found! Please install {}, refer to the README.md for help", binary_name, program))]
    DependencyMissingError { binary_name: &'static str, program: &'static str },
    #[snafu(display("The provided regex expression was faulty: {}", expr))]
    RegexError { expr: String, source: LibraryError }
}

type Result<T, E=Error> = std::result::Result<T, E>;

pub struct ThreadResult {
    pub db_entry_: DBEntry,
    pub result_: Result<Output, utils::Error>
}
impl ThreadResult {
    fn new(db_entry_: DBEntry, result_: Result<Output, utils::Error>) -> Self { Self { db_entry_, result_ } }
}

pub fn get_pw() -> Result<String> {
    let pass_gen = PasswordGenerator {
        length: 15,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        strict: true,
        exclude_similar_characters: true,
        spaces: false,
    };
    match pass_gen.generate_one() {
        Ok(pw) => return Ok(pw),
        Err(err) => return Err(Error::PasswordGeneratorError { err })
    };
}

pub fn cmd(program: &'static str, args: &[&str], port: &str) -> Result<Output> {
    let args_v = args.to_owned();
    let mut args_s = String::new();
    for arg in args_v {
        args_s.push_str(&format!(" {}", arg));
    }
    return Command::new(program)
        .args(args)
        .env("PORT", port)
        .output().context(IoError).context(CmdError {program, args:args_s});
}

//pub fn exec_nightwatch(script_path: &str, url: &str, db_entry: &DBEntry, browser_type: &String, port: &String) -> Result<Output> {
pub fn exec_nightwatch(script_path: &str, db_entry: &DBEntry, browser_type: &String, port: &String) -> Result<Output> {
    cmd("nightwatch", 
            &["--env", browser_type, "--test", script_path, 
            &db_entry.username_, &db_entry.old_password_, &db_entry.new_password_], port)
}

fn get_url_check_source_blocklist(url_: &String, blocklist: &Vec<String>, urls: &HashMap<String, String>) -> Result<String> {
    let target_url = Url::parse(&url_).context(UrlError).context(UrlParseError { url:url_.to_owned() })?;
    let target_domain = target_url.domain().ok_or(Error::UrlDomainError { url:url_.to_owned() })?.to_owned();

    if blocklist.contains(&target_domain) {
        return Err(Error::UrlDomainBlocked);
    }
    
    let mut url = target_domain.to_owned();
    for (key, value) in urls {
        let re = Regex::new(&key).context(RegexLibError).context(RegexError { expr:key })?;
        if re.is_match(&target_domain) {
            url = value.to_owned();
            break;
        }
    }

    return Ok(url);
}

pub fn get_url_and_script_path(config: &Configuration, blocklist: &Vec<String>, db_entry: &DBEntry) -> Result<String> {
    for script in config.scripts_.iter() {
        let mut script_path = PathBuf::new();
        script_path.push(&script.dir_);

        let url = get_url_check_source_blocklist(&db_entry.url_, blocklist, &config.urls_)?;
        let script_name = format!("{}.js", url);

        script_path.push(&script_name);
        let path = script_path.to_str().ok_or(Error::ScriptPathError{ url:db_entry.url_.to_owned() })?.to_owned();

        if !script_path.exists() {
            continue;
        }

        if script.blocklist_.contains(&script_name) {
            return Err(Error::ScriptBlocked);
        }
        return Ok(path);
    }
    return Err(Error::ScriptPathError{ url:db_entry.url_.to_owned() });
}

pub fn check_dependencies(config: &Configuration) -> Result<()> {
    let binary_name = "nightwatch";
    match which(binary_name) {
        Ok(_) => (),
        Err(_) => return Err(Error::DependencyMissingError { binary_name, program: "Nightwatch"})
    }
    if config.browser_type_ == BrowserType::Firefox {
        let binary_name = "firefox";
        match which(binary_name) {
            Ok(_) => (),
            Err(_) => return Err(Error::DependencyMissingError { binary_name, program: "Firefox"})

        }
    } else if config.browser_type_ == BrowserType::Chrome {
        let binary_name = "google-chrome";
        match which(binary_name) {
            Ok(_) => (),
            Err(_) => return Err(Error::DependencyMissingError { binary_name, program: "Chrome"})
        }
    }

    return Ok(());
}

pub fn check_port_available(port: u16) -> bool {
    match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => true,
        Err(_) => false
    }
}

pub fn run_update_threads(db: &DB, blocklist: &Vec<String>, config: &Configuration, tx: Sender<ThreadResult>) -> usize {
    let mut port;
    let browser_type;
    if config.browser_type_ == BrowserType::Firefox {
        port = FIREFOX_PORT;
        browser_type = "firefox".to_owned();
    } else {
        port = CHROME_PORT;
        browser_type = "chrome".to_owned();
    }
    let mut nr_jobs = 0usize;
    let pool = ThreadPool::new(config.nr_threads_);
    for db_entry in db.entries.iter() {
        let entry = db_entry.clone();
        let script_path = match get_url_and_script_path(config, blocklist, &db_entry) {
            Ok(url_path) => url_path,
            Err(utils::Error::UrlDomainBlocked) => continue,
            Err(utils::Error::ScriptBlocked) => continue,
            Err(err) => {
                eprintln!("Warning: {}", err);
                continue;
            }
        };

        let browser_type_ = browser_type.clone();
        
        nr_jobs += 1;
        let tx = tx.clone();
        pool.execute(move || {
            match exec_nightwatch(&script_path, &entry, &browser_type_, &port.to_string()) {
                Ok(output) => tx.send(ThreadResult::new(entry, Ok(output))).expect("Error: Thread could not send"),
                Err(err) => tx.send(ThreadResult::new(entry, Err(err))).expect("Error: Thread could not send")
            };
        });
        port += 1;
        while !check_port_available(port) {
            port += 1;
        }
    }
    pool.join();
    return nr_jobs;
}