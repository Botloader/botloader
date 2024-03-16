#!/bin/bash
set -e
cd "${0%/*}"

export DATABASE_URL=postgres://postgres@localhost/botloader-integration-testing

# init db 
cd ../components/stores
cargo sqlx database reset -y
cd -

RUST_LOG=info,sqlx=error cargo run --bin prepare-integration-tests -- --scripts-path ./ --guild-id '531120790318350336' $@

# run tests 
cargo run --bin backend -- full --integration-tests-guild '531120790318350336'