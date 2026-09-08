#!/usr/bin/env bash
# Regenerates wasm-pkg/ from rust-core/. Run this after any change under
# rust-core/src/ and commit the result — GitHub Pages has no build step, so
# wasm-pkg/ is generated output that must be checked in, unlike
# rust-core/target/ (gitignored).
#
# Requires: rustup target add wasm32-unknown-unknown
#           cargo install wasm-bindgen-cli --version 0.2.128 --locked
#           wasm-opt (from binaryen — `npm install -g binaryen`,
#             `apt install binaryen`, or https://github.com/WebAssembly/binaryen)
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

# -Oz: optimize for size over speed — this crate's own logic is small
# enough that the difference is noise, but the .wasm ships to every visitor
# on every load, so smaller wins. Strip-debug/producers sheds a little more
# (name/producers sections) with zero effect on behavior.
if command -v wasm-opt >/dev/null 2>&1; then
    before=$(stat -c%s wasm-pkg/rust_core_bg.wasm 2>/dev/null || stat -f%z wasm-pkg/rust_core_bg.wasm)
    wasm-opt -Oz --strip-debug --strip-producers \
        wasm-pkg/rust_core_bg.wasm -o wasm-pkg/rust_core_bg.wasm
    after=$(stat -c%s wasm-pkg/rust_core_bg.wasm 2>/dev/null || stat -f%z wasm-pkg/rust_core_bg.wasm)
    echo "wasm-opt: ${before} -> ${after} bytes"
else
    echo "wasm-opt not found — skipping size optimization (install binaryen for a smaller build)" >&2
fi

echo "wasm-pkg/ regenerated — remember to 'git add wasm-pkg' and commit it."
