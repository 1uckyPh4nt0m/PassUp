use libaes::Cipher;
use rusqlite::{Connection, params};
use openssl::{pkcs5::pbkdf2_hmac, hash};
use crate::utils::{self, get_pw, DBEntry, DB, run_update_threads};
use crate::config::{Configuration, Source};
use snafu::{ResultExt, Snafu};
use std::sync::mpsc::channel;

#[derive(Debug, Snafu)]
enum LibraryError {
    IoError { source: std::io::Error },
    Utf8Error { source: std::str::Utf8Error },
    SqliteError { source: rusqlite::Error },
    OpensslError { source: openssl::error::ErrorStack },
    UtilsError { source: utils::Error }
}

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not open DB saved at {}, {}", file, source))]
    DBOpenError { file: String, source: LibraryError },
    #[snafu(display("Could not prepare SQL Statement, {}", source))]
    SqlStatementError { source: LibraryError },
    #[snafu(display("Could not retreive the {}, {}", row_name, source))]
    RowError { row_name: String, source: LibraryError },
    #[snafu(display("Could not query needed information, {}", source))]
    SqlQueryError { source: LibraryError },
    CredentialError,
    Pbkdf2Error { source: LibraryError },
    StringConversionError { source: LibraryError },
    LibError { source: LibraryError }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
struct Login {
    origin_url: String,
    username: String,
    password: Vec<u8>
}

pub fn run(config: &Configuration) {
    for source in &config.sources_ {
        let (db, version) = match decrypt_and_parse_db(source) {
            Ok(val) => val,
            Err(err) => {
                println!("Error: {}", err);
                continue;
            }
        };
    
        let (tx, rx) = channel();
        let nr_jobs = run_update_threads(&db, &source.blocklist_, config, tx);
    
        let mut db_vec = Vec::new();

        let thread_results = rx.iter().take(nr_jobs);
        for thread_result in thread_results {
            let output = match thread_result.result_ {
                Ok(output) => output,
                Err(err) => {
                    eprintln!("Error while executing Nightwatch: {}", err);
                    continue;
                }
            };

            let mut db_entry = thread_result.db_entry_;
            if output.status.success() == true {
                println!("Updated password on website {}, with username {}", &db_entry.url_, &db_entry.username_);
                db_vec.push(db_entry);
            } else {
                let db_entry_ = db_entry.clone();
                let err = utils::Error::NightwatchExecError { db_entry: db_entry_, output};
                eprintln!("{}", err);
                db_entry.new_password_ = db_entry.old_password_.to_owned();
                db_vec.push(db_entry);
            }
        }
        let updated_db = DB::new(db_vec);

        match update_db(&source, &updated_db, version) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error: {}", err);
                println!("{:?}", &updated_db);
            }
        };
    }
}

fn cipher(encrypt: bool, text: Vec<u8>, version: &Vec<u8>) -> Result<Vec<u8>> {
    let salt = b"saltysalt";
    let iv = [32u8; 16];
    let iterations = 1;
    let pass;
    if version == b"v10" {
        pass = b"peanuts".to_ascii_lowercase();
    } else {    //add check for gnome or kwallet
        pass = b"".to_ascii_lowercase();
    }

    let mut key = [32u8; 16];
    pbkdf2_hmac(&pass, salt, iterations, hash::MessageDigest::sha1(), &mut key).context(OpensslError).context(Pbkdf2Error)?;
    let cipher = Cipher::new_128(&key);

    let mut result = Vec::new();
    if encrypt {
        result.append(&mut version.to_vec());
        result.append(&mut cipher.cbc_encrypt(&iv, &text));
    } else {
        result = cipher.cbc_decrypt(&iv, &text); 
    }
   
    return Ok(result);
}

fn decrypt_and_parse_db(source: &Source) -> Result<(DB, Vec<u8>)> {
    let sql_db = Connection::open(&source.file_).context(SqliteError).context(DBOpenError { file: source.file_.to_owned() })?;

    let mut stmt = sql_db.prepare("SELECT action_url, username_value, password_value FROM logins").context(SqliteError).context(SqlStatementError)?;
    let login_iter = stmt.query_map([], |row| {
        Ok(Login {
            origin_url: row.get(0)?,
            username: row.get(1)?,
            password: row.get(2)?
        })
    }).context(SqliteError).context(SqlQueryError)?;

    let mut version = Vec::new();
    let mut db_vec = Vec::new();
    for login_ in login_iter {
        let login = login_.unwrap();
        if login.password.is_empty() {
            continue;
        }
        let mut encrypted_password = login.password;
        version = encrypted_password[0..3].to_ascii_lowercase();
        encrypted_password = encrypted_password[3..].to_vec(); 
        let decrypted_u8 = cipher(false, encrypted_password, &version)?;
        let password = std::str::from_utf8(&decrypted_u8).context(Utf8Error).context(StringConversionError)?;
        
        db_vec.push(DBEntry::new(login.origin_url, login.username, password.to_owned(), get_pw().context(UtilsError).context(LibError)?));
    }
    return Ok((DB::new(db_vec), version));
}

fn update_db(source: &Source, db: &DB, version: Vec<u8>) -> Result<()> {
    let sql_db = Connection::open(&source.file_).context(SqliteError).context(DBOpenError { file: source.file_.to_owned() })?;

    for entry in &db.entries {
        let password_u8 = cipher(true, entry.new_password_.as_bytes().to_vec(), &version)?;
        //let password = std::str::from_utf8(&password_u8).context(Utf8Error).context(StringConversionError)?;

        //let query = format!("UPDATE logins SET password_value = ? WHERE action_url = {} AND username_value = {}", &entry.url_, &entry.username_);
        let mut query = sql_db.prepare("UPDATE logins SET password_value = ? WHERE action_url = ? AND username_value = ?").context(SqliteError).context(SqlQueryError)?;
        //let mut stmt = sql_db.prepare(&query).context(SqliteError).context(SqlStatementError)?;
        //let _rows = stmt.query([]).context(SqliteError).context(SqlStatementError)?;
        query.execute(params![password_u8, entry.url_, entry.username_]).context(SqliteError).context(SqlQueryError)?;
        //sql_db.execute(query, [&password_u8, &entry.url_.as_bytes().to_vec(), &entry.username_.as_bytes().to_vec()]).context(SqliteError).context(SqlQueryError)?;
    }

    Ok(())
}