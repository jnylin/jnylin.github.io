// Thin loader for the compiled Rust/WASM core (see /rust-core). state.js
// owns the single PinballApi instance and all the game logic that talks to
// it — this module's only job is wiring up the wasm-bindgen import once.
import init, { PinballApi } from '../wasm-pkg/rust_core.js';

let ready = null;

export async function loadWasm() {
    if (!ready) ready = init();
    await ready;
    return PinballApi;
}
