use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::mpsc::Sender;
use std::{io, net, result, str};

use kpdb::EntryUuid;
use passwords::PasswordGenerator;
use regex::Regex;
use snafu::{ResultExt, Snafu};
use threadpool::ThreadPool;
use url::Url;
use which::which;

use crate::config::{BrowserType, Configuration};
use crate::utils;

const FIREFOX_PORT: u16 = 4444;
const CHROME_PORT: u16 = 9515;
const FIREFOX_BIN: &str = "firefox";
const CHROME_BIN: &str = "google-chrome";
const NIGHTWATCH_BIN: &str = "nightwatch";
const LOCALHOST: &str = "127.0.0.1";

#[derive(Debug, Clone, PartialEq)]
pub enum Uuid {
    None,
    Kdbx(EntryUuid),
    Pwsafe([u8; 16]),
}

#[derive(Debug, Clone)]
pub struct DBEntry {
    pub url: String,
    pub username: String,
    pub old_password: String,
    pub new_password: String,
    pub uuid: Uuid,
}

impl DBEntry {
    pub fn new(url: String, username: String, old_password: String, new_password: String) -> Self {
        Self {
            url,
            username,
            old_password,
            new_password,
            uuid: Uuid::None,
        }
    }
    pub fn empty() -> Self {
        Self {
            url: "".to_owned(),
            username: "".to_owned(),
            old_password: "".to_owned(),
            new_password: "".to_owned(),
            uuid: Uuid::None,
        }
    }
}

#[derive(Debug)]
pub struct DB {
    pub entries: Vec<DBEntry>,
}

impl DB {
    pub fn new(entries: Vec<DBEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Snafu)]
pub enum LibraryError {
    UrlError { source: url::ParseError },
    IoError { source: io::Error },
    RegexLibError { source: regex::Error },
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A new password could not be generated: {}", err))]
    PasswordGeneratorError {
        err: &'static str,
    },
    #[snafu(display("Could not execute command \'{}{}\': {}", program, args, source))]
    CmdError {
        program: &'static str,
        args: String,
        source: LibraryError,
    },
    #[snafu(display("Could not parse URL \'{}\' with {}", url, source))]
    UrlParseError {
        url: String,
        source: LibraryError,
    },
    #[snafu(display("URL does not contain a domain name: {}", url))]
    UrlDomainError {
        url: String,
    },
    UrlDomainBlocked,
    #[snafu(display(
        "Script path does not result in a valid unicode string. Skipping site: {}",
        url
    ))]
    ScriptPathError {
        url: String,
    },
    #[snafu(display("Script path \'{}\' is not present", path))]
    ScriptMissingError {
        path: String,
    },
    ScriptBlocked,
    #[snafu(display("Warning: Script for website \'{}\' with username: \'{}\' did not execute successfully\n{}", db_entry.url, db_entry.username, str::from_utf8(&output.stdout).unwrap_or("error")))]
    NightwatchExecError {
        db_entry: DBEntry,
        output: Output,
    },
    #[snafu(display(
        "The binary {} was not found! Please install {}, refer to the README.md for help",
        binary_name,
        program
    ))]
    DependencyMissingError {
        binary_name: &'static str,
        program: &'static str,
    },
    #[snafu(display("The provided regex expression was faulty: {}", expr))]
    RegexError {
        expr: String,
        source: LibraryError,
    },
}

type Result<T, E = Error> = result::Result<T, E>;

pub struct ThreadResult {
    pub db_entry: DBEntry,
    pub result: Result<Output, utils::Error>,
}
impl ThreadResult {
    fn new(db_entry: DBEntry, result: Result<Output, utils::Error>) -> Self {
        Self { db_entry, result }
    }
}

pub fn get_pw() -> Result<String> {
    let pass_gen = PasswordGenerator {
        length: 16,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: true,
        strict: true,
        exclude_similar_characters: true,
        spaces: false,
    };
    pass_gen
        .generate_one()
        .map_err(|err| Error::PasswordGeneratorError { err })
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
        .output()
        .context(IoError)
        .context(CmdError {
            program,
            args: args_s,
        });
}

//pub fn exec_nightwatch(script_path: &str, url: &str, db_entry: &DBEntry, browser_type: &String, port: &String) -> Result<Output> {
pub fn exec_nightwatch(
    script_path: &str,
    db_entry: &DBEntry,
    browser_type: &str,
    port: &str,
) -> Result<Output> {
    cmd(
        NIGHTWATCH_BIN,
        &[
            "--env",
            browser_type,
            "--test",
            script_path,
            &db_entry.username,
            &db_entry.old_password,
            &db_entry.new_password,
        ],
        port,
    )
}

fn get_url_check_source_blocklist(
    url: &str,
    blocklist: &[String],
    urls: &HashMap<String, String>,
) -> Result<String> {
    let protocol = "((https://)|(http://)).+".to_owned();
    let re_protocol = Regex::new(&protocol)
        .context(RegexLibError)
        .context(RegexError { expr: protocol })?;
    let mut url_protocol = url.to_owned();
    if !re_protocol.is_match(url) {
        url_protocol.push_str("https://");
        url_protocol.push_str(url);
    }
    let target_url = Url::parse(&url_protocol)
        .context(UrlError)
        .context(UrlParseError {
            url: url_protocol.to_owned(),
        })?;
    let mut target_domain = target_url
        .domain()
        .ok_or(Error::UrlDomainError { url: url_protocol })?
        .to_owned();
    if target_domain.starts_with("www.") {
        target_domain = target_domain.trim_start_matches("www.").to_owned();
    }

    if blocklist.contains(&target_domain) {
        return Err(Error::UrlDomainBlocked);
    }

    let mut url = target_domain.to_owned();
    for (key, value) in urls {
        let re = Regex::new(key)
            .context(RegexLibError)
            .context(RegexError { expr: key })?;
        if re.is_match(&target_domain) {
            url = value.to_owned();
            break;
        }
    }

    Ok(url)
}

pub fn get_url_and_script_path(
    config: &Configuration,
    blocklist: &[String],
    db_entry: &DBEntry,
) -> Result<String> {
    let mut path = String::new();
    for script in config.scripts.iter() {
        let mut script_path = PathBuf::new();
        script_path.push(&script.dir);

        let url = get_url_check_source_blocklist(&db_entry.url, blocklist, &config.urls)?;
        let script_name = format!("{}.js", url);

        script_path.push(&script_name);
        path = script_path
            .to_str()
            .ok_or(Error::ScriptPathError {
                url: db_entry.url.to_owned(),
            })?
            .to_owned();

        if !script_path.exists() {
            continue;
        }

        if script.blocklist.contains(&script_name) {
            return Err(Error::ScriptBlocked);
        }
        return Ok(path);
    }
    Err(Error::ScriptMissingError { path })
}

pub fn check_dependencies(config: &Configuration) -> Result<()> {
    if which(NIGHTWATCH_BIN).is_err() {
        return Err(Error::DependencyMissingError {
            binary_name: NIGHTWATCH_BIN,
            program: "Nightwatch",
        });
    }
    if config.browser_type == BrowserType::Firefox {
        if which(FIREFOX_BIN).is_err() {
            return Err(Error::DependencyMissingError {
                binary_name: FIREFOX_BIN,
                program: "Firefox",
            });
        }
    } else if config.browser_type == BrowserType::Chrome && which(CHROME_BIN).is_err() {
        return Err(Error::DependencyMissingError {
            binary_name: CHROME_BIN,
            program: "Chrome",
        });
    }

    Ok(())
}

pub fn check_port_available(port: u16) -> bool {
    net::TcpListener::bind((LOCALHOST, port)).is_ok()
}

pub fn run_update_threads(
    db: &DB,
    blocklist: &[String],
    config: &Configuration,
    tx: Sender<ThreadResult>,
) -> usize {
    let mut port;
    let browser_type;
    if config.browser_type == BrowserType::Firefox {
        port = FIREFOX_PORT;
        browser_type = BrowserType::Firefox.to_string();
    } else {
        port = CHROME_PORT;
        browser_type = BrowserType::Chrome.to_string();
    }
    let mut nr_jobs = 0usize;
    let pool = ThreadPool::new(config.nr_threads);
    for db_entry in db.entries.iter() {
        let entry = db_entry.clone();
        let script_path = match get_url_and_script_path(config, blocklist, db_entry) {
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
                Ok(output) => tx
                    .send(ThreadResult::new(entry, Ok(output)))
                    .expect("Error: Thread could not send"),
                Err(err) => tx
                    .send(ThreadResult::new(entry, Err(err)))
                    .expect("Error: Thread could not send"),
            };
        });
        port += 1;
        while !check_port_available(port) {
            port += 1;
        }
    }
    pool.join();
    nr_jobs
}
