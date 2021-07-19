use libaes::Cipher;
use rusqlite::Connection;
use openssl::{pkcs5::pbkdf2_hmac, hash};

#[derive(Debug)]
struct Login {
    origin_url: String,
    username: String,
    password: Vec<u8>
}

pub fn test() {
    //let path = "/home/gabriel/.config/google-chrome/Default/Login Data";
    let path = "/home/gabriel/.config/chromium/Default/Login Data";
    let db = Connection::open(&path).unwrap();

    let mut stmt = db.prepare("SELECT action_url, username_value, password_value FROM logins").unwrap();
    let login_iter = stmt.query_map([], |row| {
        Ok(Login {
            origin_url: row.get(0).unwrap(),
            username: row.get(1).unwrap(),
            password: row.get(2).unwrap()
        })
    }).unwrap();

    for login_ in login_iter {
        let login = login_.unwrap();
        println!("{:?}", login);
        if login.password.len() == 0 {
            continue;
        }
        let mut encrypted_password = login.password;
        encrypted_password = encrypted_password[3..].to_vec();
        let salt = b"saltysalt";
        let iv = [32u8; 16];
        let iterations = 1;
        //let pass = b"peanuts";
        let pass = b"";

        let mut key = [32u8; 16];
        pbkdf2_hmac(pass, salt, iterations, hash::MessageDigest::sha1(), &mut key).unwrap();
        let cipher = Cipher::new_128(&key);
        let decrypted_u8 = cipher.cbc_decrypt(&iv, &encrypted_password);
       
        let password = std::str::from_utf8(&decrypted_u8).unwrap();

        println!("{}", password);
    }
}