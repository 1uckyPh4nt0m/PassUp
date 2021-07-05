extern crate kpdb;
extern crate rpassword;

use rpassword::read_password;
use std::fs;
use std::str;
use crate::config::{Configuration, Source};
use crate::utils::{self, get_pw, DBEntry, DB, run_update_threads, Uuid};
use kpdb::{CompositeKey, Database, Entry};
use snafu::{ResultExt, Snafu};
use std::sync::mpsc::channel;

#[derive(Debug, Snafu)]
enum LibraryError {
    IoError { source: std::io::Error },
    KpdbError { source: kpdb::Error },
    UtilsError { source: utils::Error }
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("DB was not found on the system at location: {}", file))]
    DBNotPresent { file: String },
    #[snafu(display("No url was found for an entry"))]
    UrlMissing,
    #[snafu(display("Credentials are incomplete for website \'{}\'", url))]
    CredentialMissing { url: String },
    #[snafu(display("Could not update \'{}\' with error {}", file, source))]
    DbUpdateFailed { file: String, source: LibraryError },
    #[snafu(display("Could not open DB file \'{}\': {}", file, source))]
    OpenFailed { file: String, source: LibraryError },
    #[snafu(display("Entry has wrong uuid type"))]
    WrongUuidType,
    UtilsLibError { source: LibraryError }
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub fn run(config: &Configuration) {
    for source in &config.sources_ {
        let mut kpdb_db = match unlock_db(source) {
            Ok(db) => db,
            Err(err) => {
                eprintln!("Error: {}", err);
                continue;
            }
        };
        let db = match parse_kdbx_db(&kpdb_db) {
            Ok(db) => db,
            Err(err) => {
                eprintln!("{}", err);
                return;
            }
        };

        let (tx, rx) = channel();
        let nr_jobs = run_update_threads(&db, &source.blocklist_, config, tx);
  
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
            if output.status.success() == true {
                let mut new_entry = Entry::new();
                new_entry.set_url(&db_entry.url_);
                new_entry.set_username(&db_entry.username_);
                new_entry.set_password(&db_entry.new_password_);
                let uuid = match db_entry.uuid_ {
                    Uuid::Kdbx(id) => id,
                    _ => {
                        eprintln!("{:?}", WrongUuidType);
                        continue;
                    }
                };
                (&mut kpdb_db).root_group.remove_entry(uuid);
                (&mut kpdb_db).root_group.add_entry(new_entry);
                println!("Updated password on website {}, with username {}", db_entry.url_, db_entry.username_);
            } else {
                let db_entry_ = db_entry.clone();
                let err = utils::Error::NightwatchExecError { db_entry: db_entry_, output};
                eprintln!("{}", err);
                continue;
            }
        }
        match update_db(source, &kpdb_db) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error: {}", err);
                print_db_content(&kpdb_db);
            }
        };
    }
}

fn parse_db_entry(entry: &mut Entry) -> Result<DBEntry> {
    let url_provided = entry.url().ok_or(Error::UrlMissing)?;
    let mut url;
    if !url_provided.starts_with("https://") {
        url = "https://".to_owned();
        url.push_str(&url_provided);
    } else {
        url = url_provided.to_owned();
    }
    let username = entry.username().unwrap_or("").to_owned();
    let old_pass = entry.password().unwrap_or("").to_owned();

    if username.is_empty() || old_pass.is_empty() {
        return Err(Error::CredentialMissing { url });
    }

    return Ok(DBEntry::new(url, username, old_pass, "".to_owned(), Uuid::Kdbx(entry.uuid)));
}

fn parse_kdbx_db(db: &Database) -> Result<DB> {
    let mut entries = db.root_group.entries.clone();
    let mut db_vec = Vec::new();
    for entry in entries.iter_mut() {
        let mut db_entry = match parse_db_entry(entry) {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("Warning: {}", err);
                continue;
            }
        };

        db_entry.new_password_ = get_pw().context(UtilsError).context(UtilsLibError)?;
        db_vec.push(db_entry);
    }
    return Ok(DB::new(db_vec));
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
    let mut file = fs::File::create(&source.file_).context(IoError).context(err.clone())?;
    (&mut db).save(&mut file).context(KpdbError).context(err)?;

    println!("Finished with {}!", &source.file_);
    Ok(())
}

fn unlock_db(source: &Source) -> Result<Database> {
    let key = CompositeKey::from_password("new_db");
    let mut db = Database::new(&key);
    let mut db_password;
    let mut password_wrong = true;

    
    match fs::metadata(&source.file_) {
        Ok(_) => (),
        Err(_) => return Err(Error::DBNotPresent { file: source.file_.to_owned() })
    }

    println!("Please enter password for {} at {}", source.name_, source.file_);
    while password_wrong {
        db_password = read_password().unwrap_or("".to_owned());
        let key = CompositeKey::from_password(&db_password);
        let mut file = fs::File::open(&source.file_).context(IoError).context(OpenFailed { file: source.file_.to_owned() })?;

        db = match Database::open(&mut file, &key) {
            Ok(db) => {
                password_wrong = false;
                db
            }
            Err(kpdb::Error::CryptoError(_)) => {
                println!("Wrong password! Please try again:");
                continue;
            }
            Err(kpdb::Error::InvalidKey) => {
                println!("Wrong password! Please try again:");
                continue;
            }
            Err(err) => {
                return Err(Error::OpenFailed {
                    file: source.file_.to_owned(),
                    source: LibraryError::KpdbError { source: err },
                })
            }
        };
    }
    return Ok(db);
}
