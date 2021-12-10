use std::{fs, io, result, str};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;

use snafu::{ResultExt, Snafu};

use crate::utils::{self, run_update_threads};
use crate::config::{Configuration};


#[derive(Debug, Snafu)]
pub enum LibraryError {
    IoError { source: io::Error },
    UtilsError { source: utils::Error }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not get path to the users home directory"))]
    HomeDirError,
    #[snafu(display("Path is not a valid unicode string"))]
    PathToStrError,
    #[snafu(display("Could not read directory: \'{}\' with: {}", path, source))]
    PassStoreNotFound { path: String, source: LibraryError },
    #[snafu(display("Password generation is not working"))]
    PassGenError { source: LibraryError },
    #[snafu(display("Update failed for entry: {}, {}, {}", db_entry.url_, db_entry.username_, db_entry.new_password_))]
    PassUpdateError { db_entry: utils::DBEntry },
    CmdError  { source: LibraryError },
}

type Result<T, E=Error> = result::Result<T, E>;

pub fn run(config: &Configuration) {
    let db = match parse_pass() {
        Ok(result) => result,
        Err(err) => {
            eprintln!("{}", err);
            return;
        }
    };

    let blocklist;
    if config.sources_.is_empty() {
        blocklist = Vec::new();
    } else {
        blocklist = config.sources_[0].blocklist_.clone();
    }

    let (tx, rx) = channel();
    let nr_jobs = run_update_threads(&db, &blocklist, config, tx);

    let thread_results = rx.iter().take(nr_jobs);
    for thread_result in thread_results {
        let output = match thread_result.result_ {
            Ok(output) => output,
            Err(err) => {
                eprintln!("Warning: {}", err);
                continue;
            }
        };

        let db_entry = thread_result.db_entry_;
        if output.status.success() {
            match update_pass_entry(&db_entry) {
                Ok(()) => (),
                Err(err) => {
                    eprintln!("Warning: {}", err);
                    continue;
                }
            };
        } else {
            let db_entry_ = db_entry.clone();
            let err = utils::Error::NightwatchExecError { db_entry: db_entry_, output};
            eprintln!("{}", err);
            continue;
        }
    }
}


fn parse_pass() -> Result<utils::DB> {
    let mut path = PathBuf::new();
    let home_dir = match dirs::home_dir() {
        Some(dir) => dir,
        None => return Err(Error::HomeDirError)
    };
    path.push(home_dir);
    path.push(".password-store");
    let path_s = match path.to_str() {
        Some(path) => path.to_owned(),
        None => return Err(Error::PathToStrError)
    };

    let mut db = Vec::new();

    let outer_dir = fs::read_dir(&path).context(IoError).context(PassStoreNotFound { path:path_s })?;
    for subdir_r in outer_dir {
        let subdir_os = match subdir_r {
            Ok(subdir_os) => subdir_os.file_name(),
            Err(err) => {
                eprintln!("Warning: {}", err);
                continue;
            }
        };
        let url = match subdir_os.to_str() {
            Some(dir) => dir.to_owned(),
            None => continue
        };
        if url.starts_with('.') {
            continue;
        }
        if url.is_empty() {
            break;
        }
        path.push(&url);
        let inner_dir = match fs::read_dir(&path) {
            Ok(dir) => dir,
            Err(err) => {
                eprintln!("Warning: {}", err);
                continue;
            }
        };
        for dir_entry in inner_dir {
            let name = match dir_entry {
                Ok(name) => name.file_name(),
                Err(err) => {
                    eprintln!("Warning: {}", err);
                    continue;
                }
            };
            let username = match name.to_str() {
                Some(username) => username.to_string().replace(".gpg", ""),
                None => continue
            };
            if username.eq("") {
                continue;
            }

            let arg = format!("{}/{}", &url, &username);
            let child = match Command::new("pass").args(&["show", &arg]).output() {
                Ok(child) => child,
                Err(err) => {
                    eprintln!("Warning: {}", err);
                    continue;
                }
            };
            if !child.status.success() {
                continue;
            }

            let password = match str::from_utf8(&child.stdout) {
                Ok(pass) => pass.replace("\n", ""),
                Err(err) => {
                    eprintln!("Warning: {}", err);
                    continue;
                }
            };

            let new_password = utils::get_pw().context(UtilsError).context(PassGenError)?;
            let entry = utils::DBEntry::new(url.clone(), username, password, new_password);
            db.push(entry);
        }
    }
    Ok(utils::DB::new(db))
}

fn update_pass_entry(db_entry_: &utils::DBEntry) -> Result<()> {
    let db_entry = db_entry_.clone();
    let pass_entry = format!("{}/{}", db_entry.url_, db_entry.username_);
    let output = utils::cmd("pass", &["rm", &pass_entry], "0").context(UtilsError).context(CmdError)?;
    if !output.status.success() {
        return Err(Error::PassUpdateError { db_entry });
    }

    let pass = match Command::new("pass")
        .args(&["insert", &pass_entry])
        .stdin(Stdio::piped())
        .spawn() {
            Ok(pass) => pass,
            Err(_) => return Err(Error::PassUpdateError { db_entry })
    };

    let pass_input = format!("{}\n{}", db_entry.new_password_, db_entry.new_password_);
    let echo = match Command::new("echo")
        .arg(pass_input)
        .stdout(match pass.stdin {
            Some(stdin) => stdin,
            None => return Err(Error::PassUpdateError { db_entry })
        })
        .output() {
            Ok(echo) => echo,
            Err(_) => return Err(Error::PassUpdateError { db_entry })
        };

    if !echo.status.success() {
        return Err(Error::PassUpdateError { db_entry })
    }
    println!("Updated passwords!");

    Ok(())
}