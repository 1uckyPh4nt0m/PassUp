use std::sync::mpsc::channel;
use std::{fs, io, result};

use pwsafer::{PwsafeReader, PwsafeRecordField, PwsafeWriter};
use rpassword::read_password;
use snafu::{ResultExt, Snafu};

use crate::config::{Configuration, Source};
use crate::utils::{self, get_pw, run_update_threads, DBEntry, Uuid, DB};

type Result<T, E = Error> = result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum LibraryError {
    IoError { source: io::Error },
    UtilsError { source: utils::Error },
}

#[derive(Debug, Snafu)]
pub enum Error {
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
    #[snafu(display("Could not open DB file \'{}\': {}", file, source))]
    OpenFailed {
        file: String,
        source: LibraryError,
    },
    #[snafu(display("Could not read DB file \'{}\': {}", file, err))]
    ReaderError {
        file: String,
        err: String,
    },
    #[snafu(display("DB file \'{}\' has an invalid header", file))]
    HeaderError {
        file: String,
    },
    #[snafu(display("Failed to read field of DB \'{}\': {}", file, err))]
    ReadField {
        file: String,
        err: String,
    },
    #[snafu(display("Could not verify DB \'{}\': {}", file, err))]
    VerifyDb {
        file: String,
        err: String,
    },
    UtilsLibError {
        source: LibraryError,
    },
    #[snafu(display("Could not update \'{}\' with {}", file, source))]
    DbUpdateFailed {
        file: String,
        source: LibraryError,
    },
}

pub fn run(config: &Configuration) {
    for source in &config.sources {
        let (db, db_password, version, records) = match unlock_and_parse_db(source) {
            Ok(values) => values,
            Err(err) => {
                eprintln!("Error: {}", err);
                continue;
            }
        };

        let (tx, rx) = channel();
        let nr_jobs = run_update_threads(&db, &source.blocklist, config, tx);

        let mut updated_entries = Vec::new();
        let thread_results = rx.iter().take(nr_jobs);
        for thread_result in thread_results {
            let output = match thread_result.result {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("Error while executing Nightwatch: {}", err);
                    continue;
                }
            };

            let mut db_entry = thread_result.db_entry;
            if !output.status.success() {
                let db_entry_ = db_entry.clone();
                let err = utils::Error::NightwatchExecError {
                    db_entry: db_entry_,
                    output,
                };
                eprintln!("{}", err);
                db_entry.new_password = db_entry.old_password.to_owned();
            } else {
                println!(
                    "Updated password on website {}, with username {}",
                    db_entry.url, db_entry.username
                );
            }
            updated_entries.push(db_entry);
        }

        let updated_db = DB::new(updated_entries);
        match update_db(source, &updated_db, db_password, records, version) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error: {}", err);
                println!("{:?}", updated_db);
            }
        };
    }
}

pub fn unlock_and_parse_db(source: &Source) -> Result<(DB, String, u16, Vec<(u8, Vec<u8>)>)> {
    let mut password_wrong = true;

    println!(
        "Please enter password for {} at {}",
        source.name, source.file
    );
    let mut entry_vec = Vec::new();
    let mut record_vec = Vec::new();
    let mut version = 0;
    let mut db_password = String::new();

    while password_wrong {
        db_password = read_password().unwrap_or_else(|_| "".to_owned());
        let file = fs::File::open(&source.file)
            .context(IoError)
            .context(OpenFailed {
                file: source.file.to_owned(),
            })?;
        let breader_file = io::BufReader::new(file);
        let mut psdb = match PwsafeReader::new(breader_file, db_password.as_bytes()) {
            Ok(db) => {
                password_wrong = false;
                db
            }
            Err(err) => {
                if err.to_string().eq("Invalid password") {
                    println!("Wrong password! Please try again:");
                    continue;
                } else {
                    return Err(Error::ReaderError {
                        file: source.file.to_owned(),
                        err: err.to_string(),
                    });
                }
            }
        };

        let mut entry = DBEntry::empty();

        version = match psdb.read_version() {
            Ok(ver) => ver,
            Err(_) => {
                return Err(Error::HeaderError {
                    file: source.file.to_owned(),
                })
            }
        };

        let mut skipped_version_field = false;
        loop {
            let field = match psdb.read_field() {
                Ok(field) => field,
                Err(err) => {
                    return Err(Error::ReadField {
                        file: source.file.to_owned(),
                        err: err.to_string(),
                    })
                }
            };
            let (field_type, field_data) = match field {
                Some(pair) => pair,
                None => break,
            };
            if !skipped_version_field {
                if field_type == 0xff {
                    skipped_version_field = true;
                }
                continue;
            }

            let record = match PwsafeRecordField::new(field_type, field_data.clone()) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{}", e);
                    continue;
                }
            };
            record_vec.push((field_type, field_data));
            match &record {
                PwsafeRecordField::Url(url_provided) => entry.url = url_provided.to_owned(),
                PwsafeRecordField::Username(username) => entry.username = username.to_owned(),
                PwsafeRecordField::Password(password) => entry.old_password = password.to_owned(),
                PwsafeRecordField::Uuid(uuid) => entry.uuid = Uuid::Pwsafe(uuid.to_owned()),
                PwsafeRecordField::EndOfRecord => {
                    let mut entry_ = entry.clone();
                    if !entry_.url.is_empty()
                        && !entry_.username.is_empty()
                        && !entry_.old_password.is_empty()
                    {
                        entry_.new_password =
                            get_pw().context(UtilsError).context(UtilsLibError)?;
                        entry_vec.push(entry_);
                    }
                    entry = DBEntry::empty()
                }
                _ => (),
            };
        }
        if let Err(err) = psdb.verify() {
            return Err(Error::VerifyDb {
                file: source.file.to_owned(),
                err: err.to_string(),
            });
        }
    }
    Ok((DB::new(entry_vec), db_password, version, record_vec))
}

pub fn update_db(
    source: &Source,
    db: &DB,
    db_password: String,
    records: Vec<(u8, Vec<u8>)>,
    version: u16,
) -> Result<()> {
    let err = DbUpdateFailed {
        file: source.file.to_owned(),
    };

    let filename = source.file.to_owned();
    let filename_copy = format!("{}_copy", &filename);
    fs::rename(&filename, &filename_copy)
        .context(IoError)
        .context(err.clone())?;
    let file = io::BufWriter::new(
        fs::File::create(filename)
            .context(IoError)
            .context(err.clone())?,
    );
    let mut psdb = PwsafeWriter::new(file, 2048, db_password.as_bytes())
        .context(IoError)
        .context(err.clone())?;
    let empty = [0u8, 0];

    psdb.write_field(0x00, &[version as u8, (version >> 8) as u8])
        .context(IoError)
        .context(err.clone())?; // Version field
    psdb.write_field(0xff, &empty)
        .context(IoError)
        .context(err.clone())?; // End of header

    let mut db_entry = DBEntry::empty();

    for (record_type, mut record_data) in records {
        let record = match PwsafeRecordField::new(record_type, record_data.clone()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Warning: {}", e);
                continue;
            }
        };
        match &record {
            PwsafeRecordField::Uuid(uuid) => {
                for entry in db.entries.iter() {
                    if Uuid::Pwsafe(uuid.to_owned()) == entry.uuid {
                        db_entry = entry.to_owned();
                    }
                }
            }
            PwsafeRecordField::Password(_) => {
                record_data = db_entry.new_password.as_bytes().to_vec()
            }
            PwsafeRecordField::EndOfRecord => db_entry = DBEntry::empty(),
            _ => (),
        };
        psdb.write_field(record_type, &record_data)
            .context(IoError)
            .context(err.clone())?;
    }

    psdb.finish().context(IoError).context(err.clone())?;
    fs::remove_file(&filename_copy)
        .context(IoError)
        .context(err)?;
    Ok(())
}
