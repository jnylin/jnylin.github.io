/* tslint:disable */
/* eslint-disable */

export class PinballApi {
    free(): void;
    [Symbol.dispose](): void;
    ball_dying(): boolean;
    ball_dying_timer(): number;
    ball_present(): boolean;
    ball_save_fraction(): number;
    ball_waiting(): boolean;
    ball_x(): number;
    ball_y(): number;
    balls_left(): number;
    bumper_count(): number;
    bumper_flash(i: number): number;
    combo_fraction(): number;
    combo_multiplier(): number;
    flipper_angle(i: number): number;
    flipper_count(): number;
    flipper_is_moving_up(i: number): boolean;
    high_score(): number;
    is_ball_save_active(): boolean;
    kickback_count(): number;
    kickback_flash(i: number): number;
    launch_power(): number;
    /**
     * `high_score` should come from `localStorage` on the JS side — this
     * crate never touches storage (see `state.rs`'s doc comment).
     */
    constructor(high_score: number);
    /**
     * JSON of `NudgeEvent` or `null`.
     */
    nudge(force_x: number, force_y: number): string;
    over(): boolean;
    reset(): void;
    score(): number;
    /**
     * For the touch-drag charge gesture (`input.js`'s `touchmove` handler),
     * which sets launch power directly from drag distance instead of
     * accumulating it over time the way holding Space does in `step`.
     */
    set_launch_power(value: number): void;
    shake_x(): number;
    shake_y(): number;
    slingshot_count(): number;
    slingshot_flash(i: number): number;
    /**
     * Runs one frame. `rand_unit` should be `Math.random()` — see the
     * `state.rs`/`world.rs` doc comments for why randomness is a
     * parameter rather than a dependency. Returns a JSON array of
     * `StepEvent`s (`[]` if nothing happened this frame).
     */
    step(dt_factor: number, left_pressing: boolean, right_pressing: boolean, charging: boolean, rand_unit: number): string;
    tilted(): boolean;
    /**
     * JSON of `LaunchOutcome` or `null`.
     */
    trigger_launch(rand_unit: number): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_pinballapi_free: (a: number, b: number) => void;
    readonly pinballapi_ball_dying: (a: number) => number;
    readonly pinballapi_ball_dying_timer: (a: number) => number;
    readonly pinballapi_ball_present: (a: number) => number;
    readonly pinballapi_ball_save_fraction: (a: number) => number;
    readonly pinballapi_ball_waiting: (a: number) => number;
    readonly pinballapi_ball_x: (a: number) => number;
    readonly pinballapi_ball_y: (a: number) => number;
    readonly pinballapi_balls_left: (a: number) => number;
    readonly pinballapi_bumper_count: (a: number) => number;
    readonly pinballapi_bumper_flash: (a: number, b: number) => number;
    readonly pinballapi_combo_fraction: (a: number) => number;
    readonly pinballapi_combo_multiplier: (a: number) => number;
    readonly pinballapi_flipper_angle: (a: number, b: number) => number;
    readonly pinballapi_flipper_count: (a: number) => number;
    readonly pinballapi_flipper_is_moving_up: (a: number, b: number) => number;
    readonly pinballapi_high_score: (a: number) => number;
    readonly pinballapi_is_ball_save_active: (a: number) => number;
    readonly pinballapi_kickback_count: (a: number) => number;
    readonly pinballapi_kickback_flash: (a: number, b: number) => number;
    readonly pinballapi_launch_power: (a: number) => number;
    readonly pinballapi_new: (a: number) => number;
    readonly pinballapi_nudge: (a: number, b: number, c: number) => [number, number];
    readonly pinballapi_over: (a: number) => number;
    readonly pinballapi_reset: (a: number) => void;
    readonly pinballapi_score: (a: number) => number;
    readonly pinballapi_set_launch_power: (a: number, b: number) => void;
    readonly pinballapi_shake_x: (a: number) => number;
    readonly pinballapi_shake_y: (a: number) => number;
    readonly pinballapi_slingshot_count: (a: number) => number;
    readonly pinballapi_slingshot_flash: (a: number, b: number) => number;
    readonly pinballapi_step: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly pinballapi_tilted: (a: number) => number;
    readonly pinballapi_trigger_launch: (a: number, b: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
