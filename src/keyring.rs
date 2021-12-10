use cryptex::{self, KeyRing};
use snafu::{ResultExt, Snafu};

use cryptex::keyring::linux::LinuxOsKeyRing as OsKeyRing;

#[derive(Debug, Snafu)]
pub enum LibraryError {
    KeyError { source: cryptex::error::KeyRingError},
    Utf8Error { source: std::string::FromUtf8Error },
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not find password for Chrome/Chromium"))]
    PasswordNotPresent,
    CryptexError { source: LibraryError },
    StringConversionError { source: LibraryError },
}

type Result<T, E = Error> = std::result::Result<T, E>;


pub fn get_chrome_password() -> Result<String> {
    let secrets = OsKeyRing::peek_secret("").context(KeyError).context(CryptexError)?;
    let mut password = String::new();

    for s in secrets {
        if s.0.contains("application\": \"chrome") || s.0.contains("application\": \"chromium") {
            password = String::from_utf8(s.1.as_slice().to_vec()).context(Utf8Error).context(StringConversionError)?;
            break;
        }
    }

    if password.is_empty() {
        return Err(Error::PasswordNotPresent);
    }

    Ok(password)
}