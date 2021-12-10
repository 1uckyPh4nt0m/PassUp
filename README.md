# PassUp

Automatically update passwords from common Password Managers. Supported Password Managers are:
- [pass](https://www.passwordstore.org/)
- [KeePass](https://keepass.info/)(kdbx)
- [KeePassX](https://www.keepassx.org/)(kdbx)
- [KeePassXC](https://keepassxc.org/)(kdbx)
- [PasswordSafe](https://pwsafe.org/)(psafe3)
- [Chrome](https://www.google.com/intl/de/chrome/)(sqlite)

## Getting Started
### Prerequisites
1. Install Node.js and Node package manager. Please refer to https://nodejs.org/en/download/ for more information.

    ```
    sudo apt install nodejs npm
    ```


2. Install [Nightwatch](https://nightwatchjs.org/gettingstarted/installation/):
    ```
    npm install -g nightwatch
    ```

3. Install Browser and WebDriver:

    Either install Firefox or Chrome.

    - [Firefox](https://www.mozilla.org/de/firefox/new/)
        - [Geckodriver](https://github.com/mozilla/geckodriver/releases)
        ```
        npm install geckodriver --save-dev
        ```
    - [Chrome](https://support.google.com/chrome/answer/95346?co=GENIE.Platform%3DDesktop&hl=de)
        - [Chromedriver](https://sites.google.com/chromium.org/driver/)
        ```
        npm install chromedriver --save-dev
        ```

4. Install [Rust](https://www.rust-lang.org/tools/install):
    ```
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

5. Optional for Chrome password manager

    Install Openssl and Sqlite
    ```
    sudo apt install libssl-dev pkg-config libsqlite3-dev
    ```

### Prerequisites (Ubuntu)

The following commands can be run on Ubuntu to install required packages:

```
sudo apt install nodejs nodejs-legacy npm libssl-dev pkg-config libsqlite3-dev
npm install -g nightwatch
npm install geckodriver --save-dev
npm install chromedriver --save-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Program Usage
### Compile and Run
Compile the package and all the dependencies:
```
cargo build
```
Run the program:
```
cargo run -- [Program arguments]
```

Wether or not the browser is executed in headless mode, can be changed in *nightwatch.conf.js*. To disable headless mode comment the *'-headless'* argument out for the desired browser.
### Program Arguments
Argument | Description
-------- | -----------
-c, --config \<FILE\> | Where \<FILE\> points to the toml configuration file.
-h, --help | Prints help information
-V, --version | Prints version information

### Configuration file
Allows you to choose between the browser to be used and the password manager variant.
#### Example configuration file:

```toml
active_profile = "my-private-keepassx"
browser_type = "firefox"    #browser_type = "chrome"
nr_threads = 10     #optional default: 1

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
"([^/]{1,30}://)?[^/]+gmail.com(/$|/.*)" = "myaccount.google.com.js"
"skype.com/" = "live.com.js"
"(https://)?lichess.org(/([a-z]|[A-Z]|[0-9])*)*" = "lichess.org"
"non-domain-name-in-DB-file" = ""

[[scripts]]
dir = "development/my-custom-PassUp-scripts"
```
The configuration file has to be written and saved in the [TOML](https://toml.io/en/) format.

Allowed configuration parameters:
- browser_type: ```["firefox", "chrome"]```
- profile.type: ```["kdbx", "pass", "pwsafe", "chrome-gnome", "chrome-kde"]```

The ```[urls]``` section is used to match the correct script to any url that is provided through the password database.