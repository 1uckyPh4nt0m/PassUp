use std::sync::mpsc::channel;
use std::{fs, io, result, str};

use kpdb::{CompositeKey, Database, Entry};
use rpassword::read_password;
use snafu::{ResultExt, Snafu};

use crate::config::{Configuration, Source};
use crate::utils::{self, get_pw, run_update_threads, DBEntry, Uuid, DB};

#[derive(Debug, Snafu)]
enum LibraryError {
    IoError { source: io::Error },
    KpdbError { source: kpdb::Error },
    UtilsError { source: utils::Error },
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("DB was not found on the system at location: {}", file))]
    DBNotPresent {
        file: String,
    },
    #[snafu(display("No url was found for an entry"))]
    UrlMissing,
    #[snafu(display("Credentials are incomplete for website \'{}\'", url))]
    CredentialMissing {
        url: String,
    },
    #[snafu(display("Could not update \'{}\' with {}", file, source))]
    DbUpdateFailed {
        file: String,
        source: LibraryError,
    },
    #[snafu(display("Could not open DB file \'{}\': {}", file, source))]
    OpenFailed {
        file: String,
        source: LibraryError,
    },
    #[snafu(display("Entry has wrong uuid type"))]
    WrongUuidType,
    #[snafu(display("Could not find referenced entry"))]
    EntryReference,
    UtilsLibError {
        source: LibraryError,
    },
}

type Result<T, E = Error> = result::Result<T, E>;

pub fn run(config: &Configuration) {
    for source in &config.sources {
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
                continue;
            }
        };

        let (tx, rx) = channel();
        let nr_jobs = run_update_threads(&db, &source.blocklist, config, tx);

        let thread_results = rx.iter().take(nr_jobs);
        for thread_result in thread_results {
            let output = match thread_result.result {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("Error while executing Nightwatch: {}", err);
                    continue;
                }
            };

            let db_entry = thread_result.db_entry;
            if output.status.success() {
                let mut new_entry = Entry::new();
                new_entry.set_url(&db_entry.url);
                new_entry.set_username(&db_entry.username);
                new_entry.set_password(&db_entry.new_password);
                let uuid = match db_entry.uuid {
                    Uuid::Kdbx(id) => id,
                    _ => {
                        eprintln!("{:?}", WrongUuidType);
                        continue;
                    }
                };
                kpdb_db.root_group.remove_entry(uuid);
                kpdb_db.root_group.add_entry(new_entry);
                println!(
                    "Updated password on website {}, with username {}",
                    db_entry.url, db_entry.username
                );
            } else {
                let db_entry_ = db_entry.clone();
                let err = utils::Error::NightwatchExecError {
                    db_entry: db_entry_,
                    output,
                };
                eprintln!("{}", err);
                continue;
            }
        }
        match write_db(source, &kpdb_db) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error: {}", err);
                print_db_content(&kpdb_db);
            }
        };
    }
}

fn parse_db_entry(entry: &mut Entry) -> Result<DBEntry> {
    let url = entry.url().ok_or(Error::UrlMissing)?.to_owned();
    let username = entry.username().unwrap_or("").to_owned();
    let old_pass = entry.password().unwrap_or("").to_owned();

    if username.is_empty() || old_pass.is_empty() {
        return Err(Error::CredentialMissing { url });
    }
    let mut dbentry = DBEntry::new(url, username, old_pass, "".to_owned());
    dbentry.uuid = Uuid::Kdbx(entry.uuid);
    Ok(dbentry)
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

        db_entry.new_password = get_pw().context(UtilsError).context(UtilsLibError)?;
        db_vec.push(db_entry);
    }

    db_vec = resolve_references(&db_vec)?;

    Ok(DB::new(db_vec))
}

/// Fetch X from string “{REF:” X “}” and look up X in collection of DBEntry.
/// Then replace X by getter(db_entry) recursively until prefixes and suffixes
/// do not match anymore.
fn trim_and_substitute(
    text: &str,
    db_entries: &Vec<DBEntry>,
    getter: fn(DBEntry) -> String,
) -> Result<String> {
    let prefix = "{REF:";
    let suffix = '}';

    if !(text.starts_with(prefix) && text.ends_with(suffix)) {
        return Ok(text.to_owned());
    }

    let mid = text
        .strip_prefix(prefix)
        .unwrap()
        .strip_suffix(suffix)
        .unwrap();
    let resolved = get_ref_entry(mid, &db_entries)?;

    trim_and_substitute(&getter(resolved), db_entries, getter)
}

fn resolve_references(db_vec: &Vec<DBEntry>) -> Result<Vec<DBEntry>> {
    let mut db_vec_wo_refs = Vec::new();

    for entry in db_vec {
        let mut resolved_entry = entry.clone();
        resolved_entry.username = trim_and_substitute(&entry.username, db_vec, |e| e.username)?;
        resolved_entry.old_password =
            trim_and_substitute(&entry.old_password, db_vec, |e| e.old_password)?;
        resolved_entry.new_password =
            trim_and_substitute(&entry.new_password, db_vec, |e| e.new_password)?;
        db_vec_wo_refs.push(resolved_entry);
    }

    Ok(db_vec_wo_refs)
}

fn get_ref_entry(reference: &str, db_vec_clone: &[DBEntry]) -> Result<DBEntry> {
    let ref_vec: Vec<&str> = reference.split(|c| c == '@' || c == ':').collect();
    let text = ref_vec[2].to_owned();

    let ref_entry = find_entry(db_vec_clone, text)?;
    Ok(ref_entry.clone())
}

fn find_entry(entries: &[DBEntry], uuid: String) -> Result<&DBEntry> {
    for entry in entries {
        let current_uuid = match entry.uuid {
            Uuid::Kdbx(x) => Some(x),
            _ => None,
        }
        .ok_or(Error::EntryReference)?;
        let uuid_ = current_uuid.0.to_string().replace("-", "");
        if uuid_.eq_ignore_ascii_case(&uuid) {
            return Ok(entry);
        }
    }
    Err(Error::EntryReference)
}

fn print_db_content(db: &Database) {
    println!("DB Content:");
    let mut entries = db.root_group.entries.clone();
    for entry in entries.iter_mut() {
        let db_entry = match parse_db_entry(entry) {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        println!(
            "{}, {}, {}, {}",
            db_entry.url, db_entry.username, db_entry.old_password, db_entry.new_password
        );
    }
}

fn write_db(source: &Source, db: &Database) -> Result<()> {
    let err = DbUpdateFailed {
        file: source.file.to_owned(),
    };
    fs::remove_file(&source.file)
        .context(IoError)
        .context(err.clone())?;
    let mut file = fs::File::create(&source.file)
        .context(IoError)
        .context(err.clone())?;
    db.save(&mut file).context(KpdbError).context(err)?;

    println!("Finished with {}!", &source.file);
    Ok(())
}

fn unlock_db(source: &Source) -> Result<Database> {
    let key = CompositeKey::from_password("new_db");
    let mut db = Database::new(&key);
    let mut db_password;
    let mut password_wrong = true;

    match fs::metadata(&source.file) {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::DBNotPresent {
                file: source.file.to_owned(),
            })
        }
    }

    println!(
        "Please enter password for {} at {}",
        source.name, source.file
    );
    while password_wrong {
        db_password = read_password().unwrap_or_else(|_| "".to_owned());
        let key = CompositeKey::from_password(&db_password);
        let mut file = fs::File::open(&source.file)
            .context(IoError)
            .context(OpenFailed {
                file: source.file.to_owned(),
            })?;

        db = match Database::open(&mut file, &key) {
            Ok(db) => {
                password_wrong = false;
                db
            }
            Err(kpdb::Error::CryptoError(_)) | Err(kpdb::Error::InvalidKey) => {
                println!("Wrong password! Please try again:");
                continue;
            }
            Err(err) => {
                return Err(Error::OpenFailed {
                    file: source.file.to_owned(),
                    source: LibraryError::KpdbError { source: err },
                });
            }
        };
    }
    Ok(db)
}
