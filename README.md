# PassUp

Automatically update passwords from common Password Managers. Supported Password Managers are:
- pass
- KeePass(kdbx)
- KeePassX(kdbx)
- KeePassXC(kdbx)

## Getting Started
### Prerequisites

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

[[sources]]
name = "private-kdbx"
file = "tests/resources/test_db.kdbx"
blocklist = [ "google.com", "no-password-but-note" ]

[[scripts]]
dir = "./scripts"
blocklist = [ "live.com.js" ]

[urls]
"([^/]{1,30}://)?[^/]+gmail.com(/$|/.*)" = "google.com.js"
"skype.com/" = "live.com.js"
"non-domain-name-in-DB-file" = ""

[[scripts]]
dir = "development/my-custom-PassUp-scripts"
```

Store the config file as a *.toml* file.

## Program Arguments
The program takes one argument which is the path to the config file.