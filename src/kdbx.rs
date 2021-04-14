extern crate kpdb;
extern crate rpassword;

use rpassword::read_password;
use std::fs::File;
use std::str;

use crate::config::{Configuration, Source};
use crate::utils::{exec_script, get_pw, DBEntry, DB};
use kpdb::{CompositeKey, Database, Entry};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum OtherError {
    IoError { source: std::io::Error },
    KpdbError { source: kpdb::Error },
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("No url was found for an entry"))]
    UrlMissing,
    #[snafu(display("Credentials are incomplete for site {}", url))]
    CredentialMissing { url: String },
    #[snafu(display("Could not update {} with error {}", file, source))]
    DbUpdateFailed { file: String, source: OtherError },
    #[snafu(display("Error while opening DB file {}: {}", file, source))]
    OpenFailed { file: String, source: OtherError },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn run(config: &Configuration) {
    for source in &config.sources_ {
        let mut kpdb_db = match unlock_db(source) {
            Ok(db) => db,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };
        let db = parse_kdbx_db(&kpdb_db);

        for db_entry in db.entries {
            for script in &config.scripts_ {
                let output = match exec_script(script, &source.blocklist_, &db_entry) {
                    Some(output) => output,
                    None => continue,
                };

                if output.status.success() == true {
                    let mut new_entry = Entry::new();
                    new_entry.set_url(&db_entry.url_);
                    new_entry.set_username(&db_entry.username_);
                    new_entry.set_password(&db_entry.new_password_);
                    (&mut kpdb_db).root_group.remove_entry(db_entry.uuid);
                    (&mut kpdb_db).root_group.add_entry(new_entry);
                } else {
                    eprintln!("Could not update password for site: {} and username: {}", db_entry.url_, db_entry.username_);
                    eprintln!("{}\n{}\n{}", output.status, str::from_utf8(&output.stdout).unwrap_or("error"), str::from_utf8(&output.stderr).unwrap_or("error"));
                    continue;
                }
            }
        }
        match update_db(source, &kpdb_db) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{}", err);
                print_db_content(&kpdb_db);
            }
        };
    }
}

fn parse_db_entry(entry: &mut Entry) -> Result<DBEntry> {
    let url = entry.url().unwrap_or("").to_owned();
    let username = entry.username().unwrap_or("").to_owned();
    let old_pass = entry.password().unwrap_or("").to_owned();

    if url.is_empty() {
        return Err(Error::UrlMissing);
    }
    if username.is_empty() || old_pass.is_empty() {
        return Err(Error::CredentialMissing { url });
    }

    return Ok(DBEntry::new(url, username, old_pass, "".to_owned(), entry.uuid));
}

fn parse_kdbx_db(db: &Database) -> DB {
    let mut entries = db.root_group.entries.clone();
    let mut db_vec = Vec::new();
    for entry in entries.iter_mut() {
        let mut db_entry = match parse_db_entry(entry) {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("{}", err);
                continue;
            }
        };

        db_entry.new_password_ = get_pw();
        db_vec.push(db_entry);
    }
    return DB::new(db_vec);
}

fn print_db_content(db: &Database) {
    println!("DB Content:");
    let mut entries = db.root_group.entries.clone();
    for entry in entries.iter_mut() {
        let db_entry = match parse_db_entry(entry) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        println!("{}, {}, {}, {}", db_entry.url_, db_entry.username_, db_entry.old_password_, db_entry.new_password_);
    }
}

fn update_db(source: &Source, db_: &Database) -> Result<()> {
    let mut db = db_.clone();
    let err = DbUpdateFailed {
        file: source.file_.to_owned(),
    };
    std::fs::remove_file(&source.file_).context(IoError).context(err.clone())?;
    let mut file = File::create(&source.file_).context(IoError).context(err.clone())?;
    (&mut db).save(&mut file).context(KpdbError).context(err)?;

    println!("Updated {}!", &source.file_);
    Ok(())
}

fn unlock_db(source: &Source) -> Result<Database> {
    let key = CompositeKey::from_password("new_db");
    let mut db = Database::new(&key);
    let mut db_password;
    let mut password_wrong = true;

    println!("Please enter password for {} at {}", source.name_, source.file_);
    while password_wrong {
        db_password = read_password().unwrap_or("".to_owned());
        let key = CompositeKey::from_password(&db_password);
        let mut file = File::open(&source.file_).context(IoError).context(OpenFailed { file: source.file_.to_owned() })?;

        db = match Database::open(&mut file, &key) {
            Ok(db) => {
                password_wrong = false;
                db
            }
            Err(kpdb::Error::InvalidKey) => {
                println!("Wrong password! Please try again:");
                continue;
            }
            Err(err) => {
                return Err(Error::OpenFailed {
                    file: source.file_.to_owned(),
                    source: OtherError::KpdbError { source: err },
                })
            }
        };
    }
    return Ok(db);
}
