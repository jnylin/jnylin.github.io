#!/usr/bin/env bash
# Regenerates wasm-pkg/ from rust-core/. Run this after any change under
# rust-core/src/ and commit the result — GitHub Pages has no build step, so
# wasm-pkg/ is generated output that must be checked in, unlike
# rust-core/target/ (gitignored).
#
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version 0.2.128 --locked
# (the wasm-bindgen-cli version MUST match the wasm-bindgen crate version in
# rust-core/Cargo.lock exactly, or the generated JS glue won't load.)
set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown --manifest-path rust-core/Cargo.toml

wasm-bindgen \
    --target web \
    --out-dir wasm-pkg \
    --out-name rust_core \
    rust-core/target/wasm32-unknown-unknown/release/rust_core.wasm

echo "wasm-pkg/ regenerated — remember to 'git add wasm-pkg' and commit it."
