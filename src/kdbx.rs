extern crate kpdb;
extern crate rpassword;

use std::fs::{File};
use std::str;
use rpassword::read_password;


use kpdb::{CompositeKey, Database, Entry};
use crate::utils::{DB, DBEntry, exec_script, get_pw};
use crate::config::{Configuration, Source};


pub fn run(config: &Configuration) {
    for source in &config.sources_ {
        let mut kpdb_db = unlock_db(source);
        let db = parse_kdbx_db(&kpdb_db);
            
        for db_entry in db.entries {
            for script in &config.scripts_ {
                let output = match exec_script(source, script, &db_entry) {
                    Some(output) => output,
                    None => continue
                };

                if output.status.success() == true {
                    let mut new_entry = Entry::new();
                    new_entry.set_url(&db_entry.url_);
                    new_entry.set_username(&db_entry.username_);
                    new_entry.set_password(&db_entry.new_password_);
                    (&mut kpdb_db).root_group.remove_entry(db_entry.uuid);
                    (&mut kpdb_db).root_group.add_entry(new_entry);
                } else {
                    eprintln!("Could not update password for site {} and username {}!", db_entry.url_, db_entry.username_);
                    eprintln!("{}\n{}\n{}", output.status, str::from_utf8(&output.stdout).unwrap_or("error"), str::from_utf8(&output.stderr).unwrap_or("error"));
                    continue;
                }
            }
        }
        update_db(source, kpdb_db)
    }
}

fn parse_db_entry(entry: &mut Entry) -> DBEntry {
    let url = entry.url().unwrap_or("").to_owned();
    let username = entry.username().unwrap_or("").to_owned();
    let old_pass = entry.password().unwrap_or("").to_owned();
    
    return DBEntry::new(url, username, old_pass, "".to_owned(), entry.uuid);
}

fn parse_kdbx_db(db: &Database) -> DB {
    let mut entries = db.root_group.entries.clone();
    let mut db_vec = Vec::new();
    for entry in entries.iter_mut() {
        let mut db_entry = parse_db_entry(entry);
        if db_entry.url_.eq("") {
            eprintln!("No url was found for an entry!");
            continue;
        } else if db_entry.username_.eq("") {
            eprintln!("No username was found for site {}!", db_entry.url_);
            continue;
        } else if db_entry.old_password_.eq("") {
            eprintln!("No password was found for site {}!", db_entry.url_);
            continue;
        }
        db_entry.new_password_ = get_pw();
        db_vec.push(db_entry);
    }
    return DB::new(db_vec);
}

fn print_db_content(db: &Database) {
    println!("DB Content:");
    let mut entries = db.root_group.entries.clone();
    for entry in entries.iter_mut() {
        let db_entry = parse_db_entry(entry);
        println!("{}, {}, {}, {}", db_entry.url_, db_entry.username_, db_entry.old_password_, db_entry.new_password_);
    }
}

fn update_db(source: &Source, db_: Database) {
    let mut db = db_;
    match std::fs::remove_file(&source.file_) {
        Ok(_) => (),
        Err(e) => { 
            print_db_content(&db);
            eprintln!("Could not update {} with error {}", source.file_, e); 
            return;
        }
    };
    let mut file = match File::create(&source.file_) {
        Ok(file) => file,
        Err(e) => { 
            print_db_content(&db);
            eprintln!("Could not update {} with error {}", source.file_, e); 
            return;
        }
    };
    match (&mut db).save(&mut file) {
        Ok(_) => (),
        Err(e) => { 
            print_db_content(&db);
            eprintln!("Could not update {} with error {}", source.file_, e); 
            return;
        }
    };
    println!("Updated {}!", source.file_);
}

fn unlock_db(source: &Source) -> Database {
    let key = CompositeKey::from_password("new_db");
    let mut db = Database::new(&key);
    let mut db_password;
    let mut password_wrong = true;

    println!("Please enter password for {} at {}", source.name_, source.file_);
    while password_wrong {
        db_password = read_password().unwrap_or("".to_owned());
        let key = CompositeKey::from_password(&db_password);
        let mut file = match File::open(&source.file_) {
            Ok(file) => file,
            Err(e) => { 
                eprintln!("Error while reading DB file: {}", e);
                continue;
            }
        };
        db = match Database::open(&mut file, &key) {
            Ok(db) => {
                password_wrong = false;
                db
            },
            Err(_) => {
                eprintln!("Wrong password! Please try again:");
                continue;
            }
        };
    }
    return db;
}