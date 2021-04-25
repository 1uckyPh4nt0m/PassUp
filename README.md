# PassUp

Automatically update passwords from common Password Managers. Supported Password Managers are:
- pass
- KeePass(kdbx)
- KeePassX(kdbx)
- KeePassXC(kdbx)

## Getting Started
### Prerequisites
Install Node.js. Please refer to https://nodejs.org/en/download/ for more information.

Install Nightwatch(https://nightwatchjs.org/gettingstarted/installation/):
```
npm install -g nightwatch
```

Install Browser and WebDriver:

Either install Firefox or Chrome.
- Firefox(https://www.mozilla.org/de/firefox/new/)
- Geckodriver
```
npm install geckodriver --save-dev
```
- Chrome(https://support.google.com/chrome/answer/95346?co=GENIE.Platform%3DDesktop&hl=de)
- Chromedriver
```
npm install chromedriver --save-dev
```

Install Rust(https://www.rust-lang.org/tools/install):
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Compile the package and all the dependencies:
```
cargo build
```
Run the program:
```
cargo run -- [Program arguments]
```

## Program Arguments
-c, --config <FILE>     Where <FILE> points to the toml configuration file.

-h, --help              Prints help information

-V, --version           Prints version information

## Configuration file
Allows you to choose between the browser to be used, the pass and kdbx variant.
### Example configuration file:
```
active_profile = "my-private-keepassx"
browser_type = "firefox"    #browser_type = "chrome"

[profile.my-private-keepassx]
type = "kdbx"
sources = [ "private-kdbx" ]

[profile.work-pass]
type = "pass"
sources = [ "work-pass" ]   #optional

[[sources]]
name = "private-kdbx"
file = "tests/resources/test_db.kdbx"
blocklist = [ "google.com", "yahoo.com" ]    #optional

[[sources]]
name = "work-pass"
blocklist = [ "google.com" ]    #optional

[[scripts]]
dir = "./scripts"
blocklist = [ "live.com.js" ]   #optional

[urls]  #optional
"([^/]{1,30}://)?[^/]+gmail.com(/$|/.*)" = "google.com.js"
"skype.com/" = "live.com.js"
"non-domain-name-in-DB-file" = ""

[[scripts]]
dir = "development/my-custom-PassUp-scripts"
```

Store the config file as a *.toml* file.