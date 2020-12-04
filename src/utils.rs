use std::process::{Command, Output};

extern crate passwords;
use passwords::PasswordGenerator;

pub fn get_pw() -> String {
    let pg = PasswordGenerator {
        length: 15,
        numbers: true,
        lowercase_letters: true,
        uppercase_letters: true,
        symbols: false,
        strict: true,
        exclude_similar_characters: true,
        spaces: false,
    };

    return pg.generate_one().unwrap();
}

pub fn cmd(program: &str, args: &[&str]) -> Output {
    return Command::new(program)
        .args(args)
        .output()
        .expect("failed to execute process");
}

pub fn exec_nightwatch(url: &str, username: &str, old_pass: &str, new_pass: &str) -> Output {
    let output = cmd("nightwatch", 
            &["--headless", "--env", "firefox", "--test", "scripts/lichess.js", 
            url, username, old_pass, new_pass]);
    return output;
}