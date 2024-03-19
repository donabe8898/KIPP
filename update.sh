#!/bin/sh

cargo clean

cargo build --release && cp ./help.txt ./target/release && cp ./.env ./target/release