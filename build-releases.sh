#!/bin/bash

cargo build --release --features "arch-based"
cross build --target x86_64-unknown-linux-gnu --release --features "debian-based"
cross build --target aarch64-unknown-linux-gnu --release --bin boat --bin server --bin launcher --features "rasp-based"