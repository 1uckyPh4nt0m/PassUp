extern crate passwords;
extern crate toml;
extern crate url;

use std::process::{Command, Output};
use kpdb::EntryUuid;
use passwords::PasswordGenerator;
use std::path::PathBuf;
use url::{Url};
use crate::config::{Script};
use std::io;
use snafu::{ResultExt, Snafu};


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

#[derive(Debug, Snafu)]
pub enum LibraryError {
    UrlError { source: url::ParseError },
    IoError { source: io::Error }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("A new password could not be generated: C{}", err))]
    PasswordGeneratorError { err: &'static str },
    #[snafu(display("Could not execute command {}{}: {}", program, args, source))]
    CmdError { program: &'static str, args: String, source: LibraryError },
    #[snafu(display("Could not parse URL {} with error {}", url, source))]
    UrlParseError { url: String, source: LibraryError },
    #[snafu(display("URL does not contain a domain name: {}", url))]
    UrlDomainError { url: String },
    UrlDomainBlocked,
    #[snafu(display("Script path does not result in a valid unicode string. Skipping site: {}", url))]
    ScriptPathError { url: String },
    #[snafu(display("Script path {} is not present", path))]
    ScriptMissingError { path: String },
    ScriptBlocked,
}

type Result<T, E=Error> = std::result::Result<T, E>;

//TODO let user select parameters
pub fn get_pw() -> Result<String> {
    let pass_gen = PasswordGenerator {
        length: 15,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: false,
        strict: true,
        exclude_similar_characters: true,
        spaces: false,
    };
    match pass_gen.generate_one() {
        Ok(pw) => return Ok(pw),
        Err(err) => return Err(Error::PasswordGeneratorError { err })
    };
}

pub fn cmd(program: &'static str, args: &[&str]) -> Result<Output> {
    let args_v = args.to_owned();
    let mut args_s = String::new();
    for arg in args_v {
        args_s.push_str(&format!(" {}", arg));
    }
    return Command::new(program)
        .args(args)
        .output().context(IoError).context(CmdError {program, args:args_s});
}

pub fn exec_nightwatch(script_path: &str, db_entry: &DBEntry, browser_type: &String) -> Result<Output> {
    cmd("nightwatch", 
            &["--env", browser_type, "--test", script_path, 
            &db_entry.url_, &db_entry.username_, &db_entry.old_password_, &db_entry.new_password_])
}

pub fn get_script_name_check_blocklist(url: &String, blocklist: &Vec<String>) -> Result<String> {
    let mut url_;
    if !url.contains("https://") {
        url_ = "https://".to_owned();
        url_.push_str(&url);
    } else {
        url_ = url.to_owned();
    }
    
    let target_url = Url::parse(&url_).context(UrlError).context(UrlParseError { url:url_ })?;

    let mut target_domain = target_url.domain().ok_or(Error::UrlDomainError { url:url_ })?.to_owned();

    if blocklist.contains(&target_domain) {
        return Err(Error::UrlDomainBlocked);
    }

    target_domain.push_str(".js");

    return Ok(target_domain);
}

pub fn get_script_path(script: &Script, blocklist: &Vec<String>, db_entry: &DBEntry) -> Result<String> {
    let mut script_path = PathBuf::new();
    script_path.push(&script.dir_);

    let script_name = get_script_name_check_blocklist(&db_entry.url_, blocklist)?;

    script_path.push(&script_name);
    let path = script_path.to_str().ok_or(Error::ScriptPathError{ url:db_entry.url_ })?.to_owned();

    if !script_path.exists() {
        return Err(Error::ScriptMissingError{ path });
    }

    if script.blocklist_.contains(&path) {
        return Err(Error::ScriptBlocked);
    }

    return Ok(path)
}

pub fn exec_script(script: &Script, blocklist: &Vec<String>, db_entry: &DBEntry, browser_type: &String) -> Result<Output> {
    let script_path = get_script_path(script, blocklist, &db_entry)?;

    exec_nightwatch(&script_path, &db_entry, browser_type)
}