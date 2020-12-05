extern crate kpdb;

use std::fs;
use std::fs::{File};
use std::str;
use std::path::PathBuf;


use kpdb::{CompositeKey, Database};

use crate::utils::{get_pw, exec_nightwatch};

pub fn run(path: String, db_password: String, script_dir: String, blacklist: Vec<String>) {
    let mut file = File::open(path).ok().unwrap();
    let key = CompositeKey::from_password(db_password);
    let mut db = Database::open(&mut file, &key).unwrap();
    let mut entries = db.root_group.entries.clone();
    for entry in entries.iter_mut() {
        let username = entry.username().unwrap();
        let url = entry.url().unwrap();
        let old_pass = entry.password().unwrap();
        let new_pass = &get_pw();
        if blacklist.contains(&url.to_string()) {
            continue;
        }
        println!("Entry '{}': '{}' : '{}' : '{}'", url, username, old_pass, new_pass);

        let dir_r = fs::read_dir(&script_dir);
        let dir = match dir_r {
            Ok(dir) => dir,
            Err(e) => panic!("Reading scripts folder: {}", e),
        };
        let mut script_path = PathBuf::new();
        script_path.push(&script_dir);
        for dir_entry_r in dir {
            let dir_entry = match dir_entry_r {
                Ok(dir_entry) => dir_entry,
                Err(e) => panic!("Reading scripts: {}", e),
            };
            let entry_is_file = match dir_entry.file_type() {
                Ok(entry_type) => entry_type.is_file(),
                Err(_) => false,
            };
            let script_name_r = dir_entry.file_name();
            let script_name = match script_name_r.to_str() {
                Some(script_name) => script_name,
                None => "",
            };
            if script_name == "" {
                continue;
            }
            let mut tmp_script_path = script_path.clone();
            tmp_script_path.push(script_name);
            if entry_is_file == true && script_name.contains(".js") {

                let output = exec_nightwatch(tmp_script_path.to_str().unwrap(), url, username, old_pass, new_pass);

                if output.status.success() == true {
                    let mut new_entry = entry.clone();
                    new_entry.set_password(new_pass);
                    db.root_group.remove_entry(entry.uuid);
                    db.root_group.add_entry(new_entry);
                    //entry.set_password(new_pass);
                    std::fs::remove_file("tests/resources/test_db.kdbx").ok();
                    let mut file = File::create("tests/resources/test_db.kdbx").ok().unwrap();
                    db.save(&mut file).ok();
                    println!("Updated password!");
                } else {
                    println!("Could not update password!");
                    println!("{}\n{}\n{}", output.status, str::from_utf8(&output.stdout).unwrap(), str::from_utf8(&output.stderr).unwrap());
                    continue;
                }
            } else {
                continue;
            }
        }
    }
}