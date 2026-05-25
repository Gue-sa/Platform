#!/bin/fish

set -x CARGO_TARGET_DIR target/release
cargo build --release --features "arch-based"

set -x CARGO_TARGET_DIR target/x86_64-unknown-linux-gnu
cross build --target x86_64-unknown-linux-gnu --release --features "debian-based"

set -x CARGO_TARGET_DIR target/aarch64-unknown-linux-gnu
cross build --target aarch64-unknown-linux-gnu --release --bin boat --bin server --bin launcher --features "rasp-based"