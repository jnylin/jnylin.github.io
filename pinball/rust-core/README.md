# rust-core

A learning-project Rust/WASM port of this game's simulation logic. `src/`
(the JS original) is still what actually ships and runs the live game —
this crate is being built up module by module alongside it, not yet wired
in as a replacement. See each module's doc comment for what it ports and
what deliberately changed in translation (DOM/storage/audio/particles
stay in JS; JS's `Math.random()` becomes an explicit parameter; a couple
of JS `setTimeout`-based timers became frame counters).

Ported so far: `constants.js`, `entities.js`, `physics.js`, `state.js`,
`combo.js`, `ballSave.js`, `main.js`'s per-frame update loop (`world.rs`).

## Test

    cargo test

## Regenerate wasm-pkg/

The compiled wasm and its JS glue live in `../wasm-pkg/` (sibling to
`src/`, imported by `../src/wasmBridge.js`). Unlike `target/`, that
directory is **committed** — GitHub Pages serves static files with no
build step, so the generated output has to already be in the repo for the
live site to load it.

After changing anything under `src/`, regenerate it from the `pinball/`
directory:

    ./build-wasm.sh

This requires `wasm-bindgen-cli` installed at the exact version pinned in
`Cargo.lock`'s `wasm-bindgen` entry (currently 0.2.128) — a mismatched CLI
version produces JS glue that fails to load:

    cargo install wasm-bindgen-cli --version 0.2.128 --locked
