#!/bin/sh

set -e

cargo check
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=tokio-tungstenite
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=tungstenite
cargo check --manifest-path=crate/async-proto/Cargo.toml --features=bytes,chrono,chrono-tz,either,enumset,git2,gix-hash,noisy_float,semver,serde_json,uuid
cargo test --manifest-path=crate/async-proto/Cargo.toml --all-features
cargo check --target=i686-pc-windows-msvc
cargo test --manifest-path=crate/async-proto/Cargo.toml --target=i686-pc-windows-msvc --all-features
cargo doc --all-features
