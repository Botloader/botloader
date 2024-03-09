#!/bin/bash
set -e
cd "${0%/*}"

export DATABASE_URL=postgres://postgres:123@localhost/botloader-integration-testing

# init db 
cd ../components/stores
cargo sqlx database reset -y
cd -

RUST_LOG=info,sqlx=error cargo run --bin prepare-integration-tests -- --scripts-path ./ --guild-id '531120790318350336' $@

# compile worker
cargo build --bin vmworker
cargo build --bin discordbroker

# run
trap "trap - SIGTERM && kill -- -$$" SIGINT SIGTERM EXIT

# broker
echo "running broker"
../target/debug/discordbroker &
sleep 3

echo "running scheduler"

# scheduler
cargo run --bin scheduler -- --integration-tests-guild '531120790318350336' --vmworker-bin-path "../target/debug/vmworker" #--guild-whitelist-disabled=true