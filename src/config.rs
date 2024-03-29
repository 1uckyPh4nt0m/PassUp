use std::collections::HashMap;
use std::{fmt, fs, io, result, usize};

use snafu::{ResultExt, Snafu};
use toml::Value;

#[derive(Debug, PartialEq)]
pub enum BrowserType {
    Firefox,
    Chrome,
}

impl fmt::Display for BrowserType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            BrowserType::Firefox => write!(f, "firefox"),
            BrowserType::Chrome => write!(f, "chrome"),
        }
    }
}

#[derive(Debug)]
pub struct Configuration {
    pub browser_type: BrowserType,
    pub nr_threads: usize,
    pub active_profile: String,
    pub profile: Profile,
    pub sources: Vec<Source>,
    pub scripts: Vec<Script>,
    pub urls: HashMap<String, String>,
}

impl Configuration {
    pub fn new(
        browser_type: BrowserType,
        nr_threads: usize,
        active_profile: String,
        profile: Profile,
        sources: Vec<Source>,
        scripts: Vec<Script>,
        urls: HashMap<String, String>,
    ) -> Self {
        Self {
            browser_type,
            nr_threads,
            active_profile,
            profile,
            sources,
            scripts,
            urls,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ProfileTypes {
    Kdbx,
    Pass,
    Pwsafe,
    ChromeG,
    ChromeK,
}

impl fmt::Display for ProfileTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ProfileTypes::Kdbx => write!(f, "kdbx"),
            ProfileTypes::Pass => write!(f, "pass"),
            ProfileTypes::Pwsafe => write!(f, "pwsafe"),
            ProfileTypes::ChromeG => write!(f, "chrome-gnome"),
            ProfileTypes::ChromeK => write!(f, "chrome-kde"),
        }
    }
}

#[derive(Debug)]
pub struct Profile {
    pub ptype: ProfileTypes,
    pub sources: Vec<String>,
}

impl Profile {
    pub fn new(ptype: ProfileTypes, sources: Vec<String>) -> Self {
        Self { ptype, sources }
    }
}

#[derive(Debug)]
pub struct Source {
    pub name: String,
    pub file: String,
    pub blocklist: Vec<String>,
}

impl Source {
    pub fn new(name: String, file: String, blocklist: Vec<String>) -> Self {
        Self {
            name,
            file,
            blocklist,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Script {
    pub dir: String,
    pub blocklist: Vec<String>,
}

impl Script {
    pub fn new(dir: String, blocklist: Vec<String>) -> Self {
        Self { dir, blocklist }
    }
}

const ALLOWED_PROFILE_TYPES: [&str; 5] = ["kdbx", "pass", "pwsafe", "chrome-gnome", "chrome-kde"];
const PROFILE_TYPES_WITH_SOURCE: [&str; 4] = ["kdbx", "pwsafe", "chrome-gnome", "chrome-kde"];

#[derive(Debug, Snafu)]
pub enum Error {
    //*********************************************************************************
    //Config Errors
    #[snafu(display("Could not open config from \'{}\': {}", path, source))]
    ConfigOpen {
        path: String,
        source: io::Error,
    },
    #[snafu(display("Config file from \'{}\' is not a valid toml file: {}", path, source))]
    ConfigWrongFormat {
        path: String,
        source: toml::de::Error,
    },
    #[snafu(display("Wrong browser type set in config file"))]
    ConfigBrowserTypeWrong,
    #[snafu(display("Missing browser type field in config file"))]
    ConfigBrowserTypeMissing,

    //*********************************************************************************
    //Active Profile Errors
    #[snafu(display("Active profile field missing in configuration file \'{}\'", path))]
    ActiveProfileMissingField {
        path: String,
    },
    //*********************************************************************************
    //Profile Errors
    #[snafu(display("Profile field missing in config file \'{}\'", path))]
    ProfileNotFound {
        path: String,
    },
    #[snafu(display("Profile is not a valid toml table"))]
    ProfileWrongFormat,
    #[snafu(display("Active profile is not present in the config file \'{}\'", path))]
    ProfileAPNotPresent {
        path: String,
    },
    #[snafu(display("Profile type missing"))]
    ProfileTypeMissing,
    #[snafu(display(
        "Wrong profile type, choose one of the following: {:?}",
        ALLOWED_PROFILE_TYPES
    ))]
    ProfileTypeWrong,
    #[snafu(display("Profile does not contain sources"))]
    ProfileSourcesMissing,
    //*********************************************************************************
    //Sources Errors
    #[snafu(display("Sources missing in config file \'{}\'", path))]
    SourcesNotFound {
        path: String,
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
    #[snafu(display("Script dir \'{}\' not present on system!", dir))]
    ScriptsDirNotPresent {
        dir: String,
    },
}

type Result<T, E = Error> = result::Result<T, E>;

pub fn parse_config(path: &str) -> Result<Configuration> {
    let config_r = fs::read(path).context(ConfigOpen { path })?;
    let config_str = String::from_utf8(config_r).map_err(|_| Error::ConfigOpen {
        path: path.to_string(),
        source: io::Error::new(io::ErrorKind::InvalidData, "expected UTF-8 text file"),
    })?;

    let config: Value = toml::from_str(&config_str).context(ConfigWrongFormat { path })?;

    let browser_types = config
        .get("browser_type")
        .ok_or(Error::ConfigBrowserTypeMissing)?
        .as_str()
        .ok_or(Error::ConfigBrowserTypeWrong)?
        .to_owned()
        .to_ascii_lowercase();

    let browser_type;
    if browser_types == BrowserType::Firefox.to_string() {
        browser_type = BrowserType::Firefox;
    } else if browser_types == BrowserType::Chrome.to_string() {
        browser_type = BrowserType::Chrome;
    } else {
        return Err(Error::ConfigBrowserTypeWrong);
    }

    let nr_threads = config
        .get("nr_threads")
        .unwrap_or(&Value::Integer(1))
        .as_integer()
        .unwrap_or(1)
        .abs() as usize;

    let active_profilev = config
        .get("active_profile")
        .ok_or(Error::ActiveProfileMissingField {
            path: path.to_owned(),
        })?;
    let active_profile = active_profilev.to_string().replace("\"", "");

    let mut profile_v = config.get("profile").ok_or(Error::ProfileNotFound {
        path: path.to_owned(),
    })?;
    let profile_m = profile_v.as_table().ok_or(Error::ProfileWrongFormat)?;
    profile_v = profile_m
        .get(&active_profile)
        .ok_or(Error::ProfileAPNotPresent {
            path: path.to_owned(),
        })?;
    let profile = parse_profile(profile_v)?;

    let temp = Value::Array(vec![]);
    let sources_v = match config.get("sources") {
        Some(source) => source,
        None => {
            if profile.ptype == ProfileTypes::Kdbx || profile.ptype == ProfileTypes::Pwsafe {
                return Err(Error::SourcesNotFound {
                    path: path.to_owned(),
                });
            }
            &temp
        }
    };

    let sources_vec = sources_v.as_array().ok_or(Error::SourcesWrongFormat)?;

    let mut sources = Vec::new();
    for source in sources_vec {
        match parse_source(source, &profile) {
            Ok(source) => {
                if profile.sources.contains(&source.name) {
                    sources.push(source);
                }
            }
            Err(Error::SourcesIgnore) => continue,
            Err(err) => eprintln!("Warning: {}", err),
        };
    }

    let scripts_v = config.get("scripts").ok_or(Error::ScriptsNotFound)?;
    let scripts_vec = scripts_v.as_array().ok_or(Error::ScriptsWrongFormat)?;

    let mut scripts = Vec::new();
    for script in scripts_vec {
        match parse_script(script) {
            Ok(script) => scripts.push(script),
            Err(err) => eprintln!("Warning: {}", err),
        };
    }

    let urls_r = match config.get("urls") {
        Some(urls) => urls.as_table(),
        None => None,
    };
    let temp = toml::map::Map::new();
    let urls_t = match urls_r {
        Some(urls) => urls,
        None => &temp,
    };
    let mut urls = HashMap::new();
    for (k, e) in urls_t {
        urls.insert(k.to_string(), e.to_string().replace("\"", ""));
    }

    Ok(Configuration::new(
        browser_type,
        nr_threads,
        active_profile,
        profile,
        sources,
        scripts,
        urls,
    ))
}

fn create_profiletype_map() -> HashMap<String, ProfileTypes> {
    let mut profile_map = HashMap::new();
    profile_map.insert("kdbx".to_owned(), ProfileTypes::Kdbx);
    profile_map.insert("pass".to_owned(), ProfileTypes::Pass);
    profile_map.insert("pwsafe".to_owned(), ProfileTypes::Pwsafe);
    profile_map.insert("chrome-gnome".to_owned(), ProfileTypes::ChromeG);
    profile_map.insert("chrome-kde".to_owned(), ProfileTypes::ChromeK);

    profile_map
}

fn parse_profile(profile: &Value) -> Result<Profile> {
    let type_v = profile.get("type").ok_or(Error::ProfileTypeMissing)?;
    let type_s_ = type_v.to_string().replace("\"", "");

    if !ALLOWED_PROFILE_TYPES.contains(&type_s_.as_str()) {
        return Err(Error::ProfileTypeWrong);
    }

    let sources = profile.get("sources").map(|s| s.as_array()).flatten();
    let sources_v = sources.ok_or(Error::ProfileSourcesMissing)?;

    let mut sources = Vec::new();
    for s in sources_v {
        sources.push(s.to_string().replace("\"", ""));
    }

    let profile_type_map = create_profiletype_map();
    let type_ = profile_type_map
        .get(&type_s_)
        .ok_or(Error::ProfileTypeWrong)?;
    Ok(Profile::new((*type_).clone(), sources))
}

fn parse_source(source: &Value, profile: &Profile) -> Result<Source> {
    let name_v = source.get("name").ok_or(Error::SourcesNameMissing)?;
    let name = name_v.to_string().replace("\"", "");

    if !profile.sources.contains(&name) {
        return Err(Error::SourcesIgnore);
    }

    let file = match source.get("file") {
        Some(file) => file.to_string().replace("\"", ""),
        None => {
            if PROFILE_TYPES_WITH_SOURCE.contains(&profile.ptype.to_string().as_str()) {
                return Err(Error::SourcesFileMissing);
            }
            "".to_owned()
        }
    };

    let blocklist = parse_blocklist(source);

    Ok(Source::new(name, file, blocklist))
}

fn parse_script(script: &Value) -> Result<Script> {
    let dir_v = script.get("dir").ok_or(Error::ScriptsDirMissing)?;
    let dir = dir_v.to_string().replace("\"", "");

    if fs::metadata(&dir).is_err() {
        return Err(Error::ScriptsDirNotPresent { dir });
    }

    let blocklist = parse_blocklist(script);

    Ok(Script::new(dir, blocklist))
}

fn parse_blocklist(value: &Value) -> Vec<String> {
    let blocklist_r = value.get("blocklist").map(|b| b.as_array()).flatten();
    let blocklist_v = match blocklist_r {
        Some(b) => b.to_vec(),
        None => Vec::new(),
    };

    let mut blocklist = Vec::new();
    for b in blocklist_v {
        blocklist.push(b.to_string().replace("\"", ""));
    }

    blocklist
}
