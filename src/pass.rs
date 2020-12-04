use std::process::{Command, Stdio};
use std::str;
use std::path::PathBuf;
use std::fs;

use crate::utils::{get_pw, exec_nightwatch, cmd};

struct UserData {
    url_: String,
    users_: Vec<String>
}

struct PWDB {
    url_: String,
    users_: Vec<String>,
    passwords_: Vec<String>
}

pub fn run() {
    let result = parse_pass();
    if result.is_err() {
        println!("parsing failed!");
        return;
    }

    let pw_db = result.unwrap();
    for entry in pw_db {
        let url = &(String::new() + "https://" + &entry.url_);
        assert_eq!(entry.users_.len(), entry.passwords_.len());
        for i in 0..entry.users_.len() {
            let username = &entry.users_[i];
            let old_pass = &entry.passwords_[i];
            let new_pass = &get_pw();
            println!("{}, {}, {}, {}", url, username, old_pass, new_pass);
            let output = exec_nightwatch(url, username, old_pass, new_pass);
            
            if output.status.success() {
                let pass_name = &(String::new() + &entry.url_ + "/" + username);
                let output = cmd("pass", &["rm", pass_name]);
                assert!(output.status.success());
                let mut _pass = Command::new("pass")
                    .args(&["insert", pass_name])
                    .stdin(Stdio::piped())
                    .spawn()
                    .expect("pass insert failed");
                
                let _echo = Command::new("echo")
                    .arg(String::new() + new_pass + "\n" + new_pass)
                    .stdout(_pass.stdin.unwrap()) // Converted into a Stdio here
                    .output()
                    .expect("failed echo command");

                assert!(_echo.status.success());
                //_pass.wait(); // <-- why is this not working
                println!("Updated password!");
            } else {
                println!("Could not update password!");
                println!("{}\n{}\n{}", output.status, str::from_utf8(&output.stdout).unwrap(), str::from_utf8(&output.stderr).unwrap());
                continue;
            }
        }
    }
}

fn parse_pass() -> Result<Vec<PWDB>, u8> {
    let mut path = PathBuf::new();
    path.push(dirs::home_dir().unwrap());
    path.push(".password-store");
    
    let outer_dir = fs::read_dir(&path).unwrap();

    let mut db_data = Vec::new();
    for subdir_r in outer_dir {
        let subdir_os = match subdir_r {
            Ok(subdir_os) => subdir_os.file_name(),
            Err(_) => return Err(2),
        };
        let subdir = subdir_os.to_str().unwrap();
        if subdir.starts_with(".") {
            continue;
        }
        if subdir == "" {
            break;
        }
        let mut path_ = path.clone();
        path_.push(subdir);
        let mut usernames = Vec::new();
        let inner_dir = fs::read_dir(path_).unwrap();
        for dir_entry_r in inner_dir {
            let name = match dir_entry_r {
                Ok(name) => name.file_name(),
                Err(_) => return Err(3),
            };
            let name = name.to_str().unwrap();
            if name != "" {
                usernames.push(name.to_string().replace(".gpg", ""));
                //usernames.push(name.to_string());
            }
        }
        let user_data = UserData{ 
            url_: subdir.to_string(), 
            users_: usernames
        };
        db_data.push(user_data);
    }
    let mut db_data_full = Vec::new();
    for entry in db_data {
        let mut passwords = Vec::new();
        for user in &entry.users_ {
            let arg = String::new() + &entry.url_ + "/" + &user;

            let child = Command::new("pass")
                .args(&["show", &arg])
                .output()
                .unwrap();

            let password = str::from_utf8(&child.stdout).unwrap().replace("\n", "");
            passwords.push(password);
        }
        let pw_db = PWDB{url_: entry.url_, users_: entry.users_, passwords_: passwords};
        db_data_full.push(pw_db);
    }
    return Ok(db_data_full);
}