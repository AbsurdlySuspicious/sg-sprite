#!/bin/sh
[ -f Cargo.toml ] || exit 1
git submodule update --recursive --init
cargo t --release
cargo t --test regression --release -- --nocapture --ignored
rm -r target/test_out/

