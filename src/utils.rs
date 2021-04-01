extern crate passwords;
extern crate toml;
extern crate url;

use std::process::{Command, Output};
use kpdb::EntryUuid;
use passwords::PasswordGenerator;
use toml::Value;
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use url::{Url};

pub struct Configuration {
    pub active_profile_: String,
    pub profile_: Profile,
    pub sources_: Vec<Source>,
    pub scripts_: Vec<Script>,
    pub urls_: HashMap<String, String>
}

impl Configuration {
    pub fn new(active_profile_: String, profile_: Profile, sources_: Vec<Source>, scripts_: Vec<Script>, urls_: HashMap<String, String>) -> Self { Self { active_profile_, profile_, sources_, scripts_, urls_ } }
}


pub struct Profile {
    pub type_: String,
    pub sources_: Vec<String>
}

impl Profile {
    pub fn new(type_: String, sources_: Vec<String>) -> Self { Self { type_, sources_ } }
}


pub struct Source {
    pub name_: String,
    pub file_: String,
    pub blocklist_: Vec<String>
}

impl Source {
    pub fn new(name_: String, file_: String, blocklist_: Vec<String>) -> Self { Self { name_, file_, blocklist_ } }
}


pub struct Script {
    pub dir_: String,
    pub blocklist_: Vec<String>
}

impl Script {
    pub fn new(dir_: String, blocklist_: Vec<String>) -> Self { Self { dir_, blocklist_ } }
}


pub struct DBEntry {
    pub url_: String,
    pub username_: String,
    pub old_password_: String,
    pub new_password_: String,
    pub uuid: EntryUuid
}

impl DBEntry {
    pub fn new(url_: String, username_: String, old_password_: String, new_password_: String, uuid: EntryUuid) -> Self { Self { url_, username_, old_password_, new_password_, uuid } }
}


pub struct DB {
    pub entries: Vec<DBEntry>
}

impl DB {
    pub fn new(entries: Vec<DBEntry>) -> Self { Self { entries } }
}



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

pub fn exec_nightwatch(script_path: &str, db_entry: &DBEntry) -> Output {
    let output = cmd("nightwatch", 
            &["--env", "chrome", "--test", script_path, 
            &db_entry.url_, &db_entry.username_, &db_entry.old_password_, &db_entry.new_password_]);
    return output;
}

pub fn parse_config(path: &str) -> Configuration {
    let config_r = fs::read(path);
    let config_str = match config_r {
        Ok(config_str) => String::from_utf8(config_str).unwrap_or(String::new()),
        Err(e) => panic!("Reading the config file: {}", e),
    };

    let config: Value = match toml::from_str(&config_str) {
        Ok(config) => config,
        Err(_) => panic!("Config file needs to be a valid toml file!") 
    };

    let active_profile = match config.get("active_profile") {
        Some(active_profile) => active_profile.to_string().replace("\"", ""),
        None => panic!("Active profile field missing in config file {}", path)
    };

    let profile_r = match config.get("profile") {
        Some(profile_r) => profile_r,
        None => panic!("Profile field missing in config file {}", path)
    };
    let profile_t = match profile_r.as_table() {
        Some(profile_t) => profile_t,
        None => panic!("No active profile selected!")
    };
    let profile = match profile_t.get(&active_profile) {
        Some(profile) => parse_profile(profile),
        None => panic!("Active profile is not present in the config file {}", path)
    };


    let sources_r = match config.get("sources") {
        Some(sources) => sources.as_array(),
        None => panic!("Sources missing in config file!")
    };
    let sources_v = match sources_r {
        Some(sources) => sources,
        None => panic!("Sources is not a toml array!")
    };
    let mut sources = Vec::new();
    for source in sources_v {
        match parse_source(source, &profile) {
            Some(source) => sources.push(source),
            None => ()
        };
    }


    let scripts_r = match config.get("scripts") {
        Some(scripts) => scripts.as_array(),
        None => panic!("Scripts missing in config file!")
    };
    let scripts_v = match scripts_r {
        Some(scripts) => scripts,
        None => panic!("Scripts is not a toml array!")
    };
    let mut scripts = Vec::new();
    for script in scripts_v {
        match parse_script(script) {
            Some(script) => scripts.push(script),
            None => ()
        };
    }


    let urls_r = match config.get("urls") {
        Some(urls) => urls.as_table(),
        None => None
    };
    let temp = toml::map::Map::new();
    let urls_t = match urls_r {
        Some(urls) => urls,
        None => &temp
    };
    let mut urls = HashMap::new();
    for (k,e) in urls_t {
        urls.insert(k.to_string(), e.to_string().replace("\"", ""));
    }

    return Configuration::new(active_profile, profile, sources, scripts, urls);
}

fn parse_profile(profile: &Value) -> Profile{
    let type_s = match profile.get("type") {
        Some(type_s) => type_s.to_string().replace("\"", ""),
        None => panic!("Profile type missing!")
    };
    if type_s.ne("kdbx") && type_s.ne("pass") {
        panic!("Profile type has to be either \"kdbx\" or \"pass\"")
    }

    let sources_a = match profile.get("sources") {
        Some(sources) => sources.as_array(),
        None => None
    };

    let sources_v = match sources_a {
        Some(sources) => sources,
        None => panic!("Profile does not contain sources!")
    };
    let mut sources = Vec::new();
    for s in sources_v {
        sources.push(s.to_string().replace("\"", ""));
    }
   
    return Profile::new(type_s, sources);
}

fn parse_source(source: &Value, profile: &Profile) -> Option<Source> {
    let name = match source.get("name") {
        Some(name) => name.to_string().replace("\"", ""),
        None => {
            eprintln!("Missing name field in sources!");
            return None;
        }
    };
    if !profile.sources_.contains(&name) {
        return None;
    }

    let file = match source.get("file") {
        Some(file) => file.to_string().replace("\"", ""),
        None => {
            if profile.type_.eq("kdbx") {
                eprintln!("Missing file field in sources!");
                return None;
            }
            "".to_owned()
        }
    };

    let blocklist = parse_blocklist(source);

    return Some(Source::new(name, file, blocklist));
}

fn parse_script(script: &Value) -> Option<Script> {
    let dir = match script.get("dir") {
        Some(dir) => dir.to_string().replace("\"", ""),
        None => {
            eprintln!("Missing dir field in scripts!");
            return None;
        }
    };

    match fs::read_dir(&dir) {
        Ok(_) => (),
        Err(_) => {
            eprintln!("Error script dir {} not present on system!", dir);
            return None;
        }
    };

    let blocklist = parse_blocklist(script);

    return Some(Script::new(dir, blocklist.clone()));
}

fn parse_blocklist(value: &Value) -> Vec<String> {
    let blocklist_r = match value.get("blocklist") {
        Some(b) => b.as_array(),
        None => None
    };
    let blocklist_v = match blocklist_r {
        Some(b) => b.to_vec(),
        None => Vec::new()
    };
    let mut blocklist = Vec::new();
    for b in blocklist_v {
        blocklist.push(b.to_string().replace("\"", ""));
    }
    return blocklist;
}

pub fn get_script_name_check_blocklist(url: &String, source: &Source) -> Option<String> {
    let mut url_;
    if !url.contains("https://") {
        url_ = "https://".to_owned();
        url_.push_str(&url);
    } else {
        url_ = url.to_owned();
    }
    
    let target_url = match Url::parse(&url_) {
        Ok(target_url) => target_url,
        Err(_) => {
            eprintln!("Could not parse URL {}", url_);
            return None;
        }
    };

    let mut target_domain = match target_url.domain() {
        Some(domain) => domain.to_owned(),
        None => {
            eprintln!("URL does not contain a domain name: {}", url);
            return None;
        }
    };

    if source.blocklist_.contains(&target_domain.to_string()) {
        return None;
    }

    target_domain.push_str(".js");

    return Some(target_domain);
}

pub fn get_script_path(source: &Source, script: &Script, db_entry: &DBEntry) -> Option<String> {
    let mut script_path = PathBuf::new();
    script_path.push(&script.dir_);

    let script_name = match get_script_name_check_blocklist(&db_entry.url_, source) {
        Some(target) => target,
        None => return None
    };

    script_path.push(&script_name);
    let path = script_path.to_str().unwrap_or("");
    if path.eq("") {
        eprintln!("Could not unwrap script path. Skipping site \"{}\"!", db_entry.url_);
        return None;
    }

    let entry_is_file = script_path.is_file();
    if entry_is_file == false {
        eprintln!("Script {} not present!", script_path.to_str().unwrap());
        return None;
    }

    let script_path_string = match script_path.to_str() {
        Some(path) => path.to_owned(),
        None => return None
    };

    if script.blocklist_.contains(&script_path_string) {
        return None;
    }

    return Some(script_path_string)
}

pub fn exec_script(source: &Source, script: &Script, db_entry: &DBEntry) -> Option<Output> {
    let script_path = match get_script_path(source, script, &db_entry) {
        Some(path) => path,
        None => return None
    };

    return Some(exec_nightwatch(&script_path, &db_entry));
}