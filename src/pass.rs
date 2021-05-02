use std::{process::{Command, Stdio}, sync::mpsc::channel};
use std::str;
use std::path::PathBuf;
use std::fs;

use kpdb::EntryUuid;

use crate::{utils::{DB, DBEntry, cmd, get_pw, run_update_threads}};
use crate::config::{Configuration};


pub fn run(config: &Configuration) { 
    let db = match parse_pass() {
        Some(result) => result,
        None => {
            eprintln!("Parsing failed!");
            return;
        }
    };

    let blocklist;
    if config.sources_.is_empty() {
        blocklist = Vec::new();
    } else {
        blocklist = config.sources_[0].blocklist_.clone();
    }

    let (tx, rx) = channel();
    let nr_jobs = run_update_threads(&db, &blocklist, config, tx);

    let thread_results = rx.iter().take(nr_jobs);
    for thread_result in thread_results {
        let output = match thread_result.result_ {
            Ok(output) => output,
            Err(err) => {
                eprintln!("Error while executing Nightwatch: {}", err);
                continue;
            }
        };

        let db_entry = thread_result.db_entry_;
        if output.status.success() {
            update_pass_entry(&db_entry);
        } else {
            println!("Could not update password!");
            println!("{}\n{}\n{}", output.status, str::from_utf8(&output.stdout).unwrap_or("error"), str::from_utf8(&output.stderr).unwrap_or("error"));
        }
    }
}


fn parse_pass() -> Option<DB> {
    let mut path = PathBuf::new();
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => return None
    };
    path.push(home_dir);
    path.push(".password-store");
    
    let mut db = Vec::new();

    let outer_dir = match fs::read_dir(&path) {
        Ok(dir) => dir,
        Err(_) => return None
    };
    for subdir_r in outer_dir {
        let subdir_os = match subdir_r {
            Ok(subdir_os) => subdir_os.file_name(),
            Err(_) => continue,
        };
        let url = match subdir_os.to_str() {
            Some(dir) => dir.to_owned(),
            None => continue
        };
        if url.starts_with(".") {
            continue;
        }
        if url == "" {
            break;
        }
        path.push(&url);
        let inner_dir = match fs::read_dir(&path) {
            Ok(dir) => dir,
            Err(_) => continue
        };
        for dir_entry in inner_dir {
            let name = match dir_entry {
                Ok(name) => name.file_name(),
                Err(_) => continue,
            };
            let username = match name.to_str() {
                Some(username) => username.to_string().replace(".gpg", ""),
                None => continue
            };
            if username.eq("") {
                continue;
            }

            let arg = format!("{}/{}", &url, &username);
            let child = match Command::new("pass").args(&["show", &arg]).output() {
                Ok(child) => child,
                Err(_) => continue
            };
            if !child.status.success() {
                continue;
            }

            let password = match str::from_utf8(&child.stdout) {
                Ok(pass) => pass.replace("\n", ""),
                Err(_) => continue
            };
            
            let mut url_ = "https://".to_owned();
            url_.push_str(&url);
            let entry = DBEntry::new(url_.clone(), username, password, get_pw(), EntryUuid::nil());
            db.push(entry);
        }
    }
    return Some(DB::new(db));
}

fn print_entry_on_error(db_entry: &DBEntry) {
    println!("Update failed for entry: {}, {}, {}", db_entry.url_, db_entry.username_, db_entry.new_password_);
}

fn update_pass_entry(db_entry: &DBEntry) -> Option<()> {
    let url = db_entry.url_.clone().replace("https://", "");
    let pass_entry = format!("{}/{}", url, db_entry.username_);
    let output = match cmd("pass", &["rm", &pass_entry], "0") {
        Ok(output) => output,
        Err(_) => return None
    };
    if !output.status.success() {
        print_entry_on_error(&db_entry);
        return None;
    }

    let pass = match Command::new("pass")
        .args(&["insert", &pass_entry])
        .stdin(Stdio::piped())
        .spawn() {
            Ok(pass) => pass,
            Err(_) => {
                print_entry_on_error(&db_entry);
                return None;
            }
        };
    
    let pass_input = format!("{}\n{}", db_entry.new_password_, db_entry.new_password_);
    let echo = match Command::new("echo")
        .arg(pass_input)
        .stdout(match pass.stdin {
            Some(stdin) => stdin,
            None => {
                print_entry_on_error(&db_entry);
                return None;
            }
        }) // Converted into a Stdio here
        .output() {
            Ok(echo) => echo,
            Err(_) => {
                print_entry_on_error(&db_entry);
                return None;
            }
        };

    if !echo.status.success() {
        print_entry_on_error(&db_entry);
        return None;
    }
    //pass.wait(); // <-- why is this not working
    println!("Updated passwords!");

    Some(())
}