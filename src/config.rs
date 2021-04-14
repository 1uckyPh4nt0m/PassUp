use toml::Value;
use std::fs;
use std::collections::HashMap;

use snafu::{ResultExt, Snafu};

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

#[derive(Debug, Snafu)]
pub enum Error {
    //*********************************************************************************
    //Config Errors
    #[snafu(display("Could not open config from {}: {}", path, source))]
    ConfigOpen {
        path: String,
        source: std::io::Error
    },
    #[snafu(display("Config file from {} is not a valid toml file: {}", path, source))]
    ConfigWrongFormat {
        path: String,
        source: toml::de::Error
    },
    //*********************************************************************************
    //Active Profile Errors
    #[snafu(display("Active profile field missing in config file {}", path))]
    ActiveProfileMissingField {
        path: String
    },
    //*********************************************************************************
    //Profile Errors
    #[snafu(display("Profile field missing in config file {}", path))]
    ProfileNotFound {
        path: String
    },
    #[snafu(display("Profile is not a valid toml table"))]
    ProfileWrongFormat,
    #[snafu(display("Active profile is not present in the config file {}", path))]
    ProfileAPNotPresent {
        path: String
    },
    #[snafu(display("Profile type missing"))]
    ProfileTypeMissing,
    #[snafu(display("Wrong profile, has to be either \"kdbx\" or \"pass\""))]
    ProfileTypeWrong,
    #[snafu(display("Profile does not contain sources"))]
    ProfileSourcesMissing,
    //*********************************************************************************
    //Sources Errors
    #[snafu(display("Sources missing in config file {}", path))]
    SourcesNotFound {
        path: String
    },
    #[snafu(display("Sources is not a valid toml array"))]
    SourcesWrongFormat,
    #[snafu(display("Missing name field in sources"))]
    SourcesNameMissing,
    #[snafu(display("Missing file field in sources"))]
    SourcesFileMissing,
    SourcesIgnore,
    //*********************************************************************************
    //Scripts Errors
    #[snafu(display("Scripts missing in config file"))]
    ScriptsNotFound,
    #[snafu(display("Scripts is not a valid toml array"))]
    ScriptsWrongFormat,
    #[snafu(display("Missing dir field in scripts"))]
    ScriptsDirMissing,
    #[snafu(display("Error script dir {} not present on system!", dir))]
    ScriptsDirNotPresent {
        dir: String
    }
}

type Result<T, E=Error> = std::result::Result<T, E>;

pub fn parse_config(path: &str) -> Result<Configuration> {
    let config_r = fs::read(path).context(ConfigOpen { path })?;
    let config_str = String::from_utf8(config_r).unwrap_or(String::new());

    let config: Value = toml::from_str(&config_str).context(ConfigWrongFormat { path })?;

    let active_profile_v = config.get("active_profile").ok_or(Error::ActiveProfileMissingField { path:path.to_owned() })?;
    let active_profile = active_profile_v.to_string().replace("\"", "");

    let mut profile_v = config.get("profile").ok_or(Error::ProfileNotFound { path:path.to_owned() })?;
    let profile_m = profile_v.as_table().ok_or(Error::ProfileWrongFormat)?;
    profile_v = profile_m.get(&active_profile).ok_or(Error::ProfileAPNotPresent { path:path.to_owned() })?;
    let profile = parse_profile(profile_v)?;

    let temp = Value::Array(vec![]);
    let sources_v = match config.get("sources") {
        Some(source) => source,
        None => {
            if profile.type_.eq("kdbx") {
                return Err(Error::SourcesNotFound { path:path.to_owned() });
            }
            &temp
        }
    };
    
    let sources_vec = sources_v.as_array().ok_or(Error::SourcesWrongFormat)?;

    let mut sources = Vec::new();
    for source in sources_vec {
        match parse_source(source, &profile) {
            Ok(source) => {
                if profile.sources_.contains(&source.name_) {
                    sources.push(source);
                }
            }
            Err(Error::SourcesIgnore) => continue,
            Err(err) => println!("{}", err)
        };
    }

    let scripts_v = config.get("scripts").ok_or(Error::ScriptsNotFound)?;
    let scripts_vec = scripts_v.as_array().ok_or(Error::ScriptsWrongFormat)?;

    let mut scripts = Vec::new();
    for script in scripts_vec {
        match parse_script(script) {
            Ok(script) => scripts.push(script),
            Err(err) => println!("{}", err)
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

    return Ok(Configuration::new(active_profile, profile, sources, scripts, urls));
}

fn parse_profile(profile: &Value) -> Result<Profile> {
    let type_v = profile.get("type").ok_or(Error::ProfileTypeMissing)?;
    let type_s = type_v.to_string().replace("\"", "");
   
    if type_s.ne("kdbx") && type_s.ne("pass") {
        return Err(Error::ProfileTypeWrong);
    }

    let sources = profile.get("sources").map(|s| s.as_array()).flatten();
    let sources_v = sources.ok_or(Error::ProfileSourcesMissing)?;

    let mut sources = Vec::new();
    for s in sources_v {
        sources.push(s.to_string().replace("\"", ""));
    }
   
    return Ok(Profile::new(type_s, sources));
}

fn parse_source(source: &Value, profile: &Profile) -> Result<Source> {
    let name_v = source.get("name").ok_or(Error::SourcesNameMissing)?;
    let name = name_v.to_string().replace("\"", "");

    if !profile.sources_.contains(&name) {
        return Err(Error::SourcesIgnore);
    }

    let file = match source.get("file") {
        Some(file) => file.to_string().replace("\"", ""),
        None => {
            if profile.type_.eq("kdbx") {
                return Err(Error::SourcesFileMissing);
            }
            "".to_owned()
        }
    };

    let blocklist = parse_blocklist(source);

    return Ok(Source::new(name, file, blocklist));
}

fn parse_script(script: &Value) -> Result<Script> {
    let dir_v = script.get("dir").ok_or(Error::ScriptsDirMissing)?;
    let dir = dir_v.to_string().replace("\"", "");

    if fs::metadata(&dir).is_err() {
        return Err(Error::ScriptsDirNotPresent { dir });
    }

    let blocklist = parse_blocklist(script);

    return Ok(Script::new(dir, blocklist.clone()));
}

fn parse_blocklist(value: &Value) -> Vec<String> {
    let blocklist_r = value.get("blocklist").map(|b| b.as_array()).flatten();
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