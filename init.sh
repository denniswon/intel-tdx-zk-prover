#!/bin/bash
# While developing, we want to keep the database state up-to date at all times

# Kill previous instance of script
killall -s 9 tdx-prover
cargo sqlx prepare
cargo run
