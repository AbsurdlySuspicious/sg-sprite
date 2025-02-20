#!/bin/sh
[ -f Cargo.toml ] || exit 1
git submodule update --recursive --init
cargo t --locked --release
cargo t --locked --test regression --release -- --nocapture --ignored
rm -r target/test_out/

