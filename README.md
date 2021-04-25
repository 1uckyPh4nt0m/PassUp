# PassUp

Automatically update passwords from common Password Managers. Supported Password Managers are:
- pass
- KeePass(kdbx)
- KeePassX(kdbx)
- KeePassXC(kdbx)

## Getting Started
### Prerequisites
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
cargo run -- -c path_to_config_file+
```

## Program Arguments
The program takes one argument which is the path to the config file.

## Config file
Allows you to choose between pass and the kdbx variant.
### Example configuration file:
```
active_profile = "my-private-keepassx"

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