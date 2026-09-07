/* @ts-self-types="./rust_core.d.ts" */

export class PinballApi {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PinballApiFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pinballapi_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    ball_dying() {
        const ret = wasm.pinballapi_ball_dying(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    ball_dying_timer() {
        const ret = wasm.pinballapi_ball_dying_timer(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    ball_present() {
        const ret = wasm.pinballapi_ball_present(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    ball_save_fraction() {
        const ret = wasm.pinballapi_ball_save_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    ball_waiting() {
        const ret = wasm.pinballapi_ball_waiting(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    ball_x() {
        const ret = wasm.pinballapi_ball_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    ball_y() {
        const ret = wasm.pinballapi_ball_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    balls_left() {
        const ret = wasm.pinballapi_balls_left(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    bumper_count() {
        const ret = wasm.pinballapi_bumper_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} i
     * @returns {number}
     */
    bumper_flash(i) {
        const ret = wasm.pinballapi_bumper_flash(this.__wbg_ptr, i);
        return ret;
    }
    /**
     * @returns {number}
     */
    combo_fraction() {
        const ret = wasm.pinballapi_combo_fraction(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    combo_multiplier() {
        const ret = wasm.pinballapi_combo_multiplier(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} i
     * @returns {number}
     */
    flipper_angle(i) {
        const ret = wasm.pinballapi_flipper_angle(this.__wbg_ptr, i);
        return ret;
    }
    /**
     * @returns {number}
     */
    flipper_count() {
        const ret = wasm.pinballapi_flipper_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} i
     * @returns {boolean}
     */
    flipper_is_moving_up(i) {
        const ret = wasm.pinballapi_flipper_is_moving_up(this.__wbg_ptr, i);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    high_score() {
        const ret = wasm.pinballapi_high_score(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    is_ball_save_active() {
        const ret = wasm.pinballapi_is_ball_save_active(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    kickback_count() {
        const ret = wasm.pinballapi_kickback_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} i
     * @returns {number}
     */
    kickback_flash(i) {
        const ret = wasm.pinballapi_kickback_flash(this.__wbg_ptr, i);
        return ret;
    }
    /**
     * @returns {number}
     */
    launch_power() {
        const ret = wasm.pinballapi_launch_power(this.__wbg_ptr);
        return ret;
    }
    /**
     * `high_score` should come from `localStorage` on the JS side — this
     * crate never touches storage (see `state.rs`'s doc comment).
     * @param {number} high_score
     */
    constructor(high_score) {
        const ret = wasm.pinballapi_new(high_score);
        this.__wbg_ptr = ret;
        PinballApiFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * JSON of `NudgeEvent` or `null`.
     * @param {number} force_x
     * @param {number} force_y
     * @returns {string}
     */
    nudge(force_x, force_y) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.pinballapi_nudge(this.__wbg_ptr, force_x, force_y);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    over() {
        const ret = wasm.pinballapi_over(this.__wbg_ptr);
        return ret !== 0;
    }
    reset() {
        wasm.pinballapi_reset(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    score() {
        const ret = wasm.pinballapi_score(this.__wbg_ptr);
        return ret;
    }
    /**
     * For the touch-drag charge gesture (`input.js`'s `touchmove` handler),
     * which sets launch power directly from drag distance instead of
     * accumulating it over time the way holding Space does in `step`.
     * @param {number} value
     */
    set_launch_power(value) {
        wasm.pinballapi_set_launch_power(this.__wbg_ptr, value);
    }
    /**
     * @returns {number}
     */
    shake_x() {
        const ret = wasm.pinballapi_shake_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    shake_y() {
        const ret = wasm.pinballapi_shake_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    slingshot_count() {
        const ret = wasm.pinballapi_slingshot_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} i
     * @returns {number}
     */
    slingshot_flash(i) {
        const ret = wasm.pinballapi_slingshot_flash(this.__wbg_ptr, i);
        return ret;
    }
    /**
     * Runs one frame. `rand_unit` should be `Math.random()` — see the
     * `state.rs`/`world.rs` doc comments for why randomness is a
     * parameter rather than a dependency. Returns a JSON array of
     * `StepEvent`s (`[]` if nothing happened this frame).
     * @param {number} dt_factor
     * @param {boolean} left_pressing
     * @param {boolean} right_pressing
     * @param {boolean} charging
     * @param {number} rand_unit
     * @returns {string}
     */
    step(dt_factor, left_pressing, right_pressing, charging, rand_unit) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.pinballapi_step(this.__wbg_ptr, dt_factor, left_pressing, right_pressing, charging, rand_unit);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    tilted() {
        const ret = wasm.pinballapi_tilted(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * JSON of `LaunchOutcome` or `null`.
     * @param {number} rand_unit
     * @returns {string}
     */
    trigger_launch(rand_unit) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.pinballapi_trigger_launch(this.__wbg_ptr, rand_unit);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) PinballApi.prototype[Symbol.dispose] = PinballApi.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_5d9e815e6fdf150f: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./rust_core_bg.js": import0,
    };
}

const PinballApiFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_pinballapi_free(ptr, 1));

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('rust_core_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
