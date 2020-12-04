extern crate kpdb;

use std::fs::File;
use std::str;

use kpdb::{CompositeKey, Database};

use crate::utils::{get_pw, exec_nightwatch};

pub fn run(path: &str, db_password: &str) {
    let mut file = File::open(path).ok().unwrap();
    let key = CompositeKey::from_password(db_password);
    let mut db = Database::open(&mut file, &key).unwrap();
    let mut entries = db.root_group.entries.clone();
    for entry in entries.iter_mut() {
        let username = entry.username().unwrap();
        let url = entry.url().unwrap();
        let old_pass = entry.password().unwrap();
        let new_pass = &get_pw();
        println!("Entry '{0}': '{1}' : '{2}' : '{3}'", url, username, old_pass, new_pass);

        let output = exec_nightwatch(url, username, old_pass, new_pass);

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
    }
}