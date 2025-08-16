#!/bin/sh

set -e

cargo check
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=tokio-tungstenite021
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=tokio-tungstenite024
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=tokio-tungstenite027
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=bitvec,bytes,bytesize,chrono,chrono-tz,doubloon,either,enumset,git2,gix-hash,noisy_float,nonempty-collections,rust_decimal,semver,serde_json,serenity,url,uuid
cargo test --manifest-path=crate/async-proto/Cargo.toml --all-features
cargo check --target=i686-pc-windows-msvc
cargo test --manifest-path=crate/async-proto/Cargo.toml --target=i686-pc-windows-msvc --all-features
cargo doc --all-features
