#!/bin/sh

set -e

sudo apt install nodejs nodejs-legacy npm libssl-dev pkg-config libsqlite3-dev
npm install -g nightwatch
npm install geckodriver --save-dev
npm install chromedriver --save-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
