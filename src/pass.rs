use std::{process::{Command, Stdio}};
use std::str;
use std::path::PathBuf;
use std::fs;

use kpdb::EntryUuid;

use crate::utils::{get_pw, cmd, DB, DBEntry, exec_script};
use crate::config::{Configuration, Source};


pub fn run(config: &Configuration) { 
    for source in &config.sources_ {
        let db = match parse_pass(source) {
            Some(result) => result,
            None => {
                eprintln!("Parsing failed for {}!", source.name_);
                continue;
            }
        };
        
        for db_entry in db.entries {
            for script in &config.scripts_ {
                let output = match exec_script(source, script, &db_entry) {
                    Some(output) => output,
                    None => continue
                };
        
                if output.status.success() {
                    update_pass_entry(&db_entry);
                } else {
                    println!("Could not update password!");
                    println!("{}\n{}\n{}", output.status, str::from_utf8(&output.stdout).unwrap_or("error"), str::from_utf8(&output.stderr).unwrap_or("error"));
                }
            }
        }
    }
}


fn parse_pass(source: &Source) -> Option<DB> {
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
            Err(_) => return None,
        };
        let url = match subdir_os.to_str() {
            Some(dir) => dir.to_owned(),
            None => return None
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
            Err(_) => return None
        };
        for dir_entry in inner_dir {
            let name = match dir_entry {
                Ok(name) => name.file_name(),
                Err(_) => return None,
            };
            let username = match name.to_str() {
                Some(username) => username.to_string().replace(".gpg", ""),
                None => return None
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
                return None;
            }

            let password = match str::from_utf8(&child.stdout) {
                Ok(pass) => pass.replace("\n", ""),
                Err(_) => return None
            };
            
            let entry = DBEntry::new(url.clone(), username, password, get_pw(), EntryUuid::nil());
            db.push(entry);
        }
    }
    return Some(DB::new(db));
}

fn print_entry_on_error(db_entry: &DBEntry) {
    println!("Update failed for entry: {}, {}, {}", db_entry.url_, db_entry.username_, db_entry.new_password_);
}

fn update_pass_entry(db_entry: &DBEntry) -> Option<()> {
    let pass_entry = format!("{}/{}", db_entry.url_, db_entry.username_);
    let output = match cmd("pass", &["rm", &pass_entry]) {
        Some(output) => output,
        None => return None
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