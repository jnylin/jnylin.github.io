//! The actual JS/WASM boundary: a `#[wasm_bindgen]` wrapper around [`World`]
//! exposing exactly what `main.js`/`state.js`/`render.js` need each frame.
//!
//! Two shapes of API here, for two different reasons:
//!
//! - **Actions and events** (`step`, `trigger_launch`, `nudge`) return a
//!   JSON *string*. wasm-bindgen can't hand JS an `enum` or a `Vec<enum>`
//!   directly without either serializing it somehow or hand-rolling a
//!   parallel-getter-arrays API (query each field of each event by index —
//!   painful for something with as many variants as `StepEvent`). JSON via
//!   `serde` is the standard answer for exactly this in the wasm-bindgen
//!   ecosystem, so the caller just does `JSON.parse(...)` and switches on
//!   `.event`/`.type`.
//! - **Per-frame state** (score, ball position, flipper angles, ...) is
//!   exposed as plain getters instead, one call per value. That's more
//!   boilerplate than a single "give me everything" JSON blob, but it's
//!   called 60 times a second — round-tripping through JSON string
//!   allocation and parsing for data that's just a handful of numbers isn't
//!   worth it. Only the fields that actually change over time are exposed;
//!   static layout (entity positions, radii, ...) stays in `entities.js`,
//!   already loaded once on the JS side.
//!
//! Indices/counts are `u32` rather than `usize` — `usize`'s width isn't
//! fixed across targets, and wasm-bindgen wants a concrete numeric type.

use wasm_bindgen::prelude::*;

use crate::world::World;

#[wasm_bindgen]
pub struct PinballApi {
    world: World,
}

#[wasm_bindgen]
impl PinballApi {
    /// `high_score` should come from `localStorage` on the JS side — this
    /// crate never touches storage (see `state.rs`'s doc comment).
    #[wasm_bindgen(constructor)]
    pub fn new(high_score: f64) -> PinballApi {
        PinballApi { world: World::new(high_score as i64) }
    }

    /// Runs one frame. `rand_unit` should be `Math.random()` — see the
    /// `state.rs`/`world.rs` doc comments for why randomness is a
    /// parameter rather than a dependency. Returns a JSON array of
    /// `StepEvent`s (`[]` if nothing happened this frame).
    pub fn step(
        &mut self,
        dt_factor: f64,
        left_pressing: bool,
        right_pressing: bool,
        charging: bool,
        rand_unit: f64,
    ) -> String {
        let events = self.world.step(dt_factor, left_pressing, right_pressing, charging, rand_unit);
        serde_json::to_string(&events).unwrap_or_else(|_| "[]".to_string())
    }

    /// JSON of `LaunchOutcome` or `null`.
    pub fn trigger_launch(&mut self, rand_unit: f64) -> String {
        serde_json::to_string(&self.world.trigger_launch(rand_unit)).unwrap_or_else(|_| "null".to_string())
    }

    /// JSON of `NudgeEvent` or `null`.
    pub fn nudge(&mut self, force_x: f64, force_y: f64) -> String {
        serde_json::to_string(&self.world.nudge(force_x, force_y)).unwrap_or_else(|_| "null".to_string())
    }

    pub fn reset(&mut self) {
        self.world.reset();
    }

    // --- Game state ---

    pub fn score(&self) -> f64 {
        self.world.state.score as f64
    }
    pub fn high_score(&self) -> f64 {
        self.world.state.high_score as f64
    }
    pub fn balls_left(&self) -> i32 {
        self.world.state.balls_left
    }
    pub fn over(&self) -> bool {
        self.world.state.over
    }
    pub fn tilted(&self) -> bool {
        self.world.state.tilted
    }
    pub fn launch_power(&self) -> f64 {
        self.world.state.launch_power
    }
    /// For the touch-drag charge gesture (`input.js`'s `touchmove` handler),
    /// which sets launch power directly from drag distance instead of
    /// accumulating it over time the way holding Space does in `step`.
    pub fn set_launch_power(&mut self, value: f64) {
        self.world.state.launch_power = value.clamp(0.0, 1.0);
    }
    pub fn shake_x(&self) -> f64 {
        self.world.state.shake.x
    }
    pub fn shake_y(&self) -> f64 {
        self.world.state.shake.y
    }
    pub fn is_ball_save_active(&self) -> bool {
        self.world.state.is_ball_save_active()
    }
    pub fn ball_save_fraction(&self) -> f64 {
        self.world.state.ball_save_fraction()
    }
    pub fn combo_multiplier(&self) -> f64 {
        self.world.state.combo_multiplier()
    }
    pub fn combo_fraction(&self) -> f64 {
        self.world.state.combo_fraction()
    }

    // --- Ball ---

    pub fn ball_present(&self) -> bool {
        self.world.state.ball.is_some()
    }
    pub fn ball_x(&self) -> f64 {
        self.world.state.ball.map(|b| b.x).unwrap_or(0.0)
    }
    pub fn ball_y(&self) -> f64 {
        self.world.state.ball.map(|b| b.y).unwrap_or(0.0)
    }
    pub fn ball_dying(&self) -> bool {
        self.world.state.ball.map(|b| b.dying).unwrap_or(false)
    }
    pub fn ball_dying_timer(&self) -> i32 {
        self.world.state.ball.map(|b| b.dying_timer).unwrap_or(0)
    }
    pub fn ball_waiting(&self) -> bool {
        self.world.state.ball.map(|b| b.waiting).unwrap_or(false)
    }

    // --- Entities: only the fields that mutate frame-to-frame. Positions,
    // radii, endpoints etc. never change after construction and stay in
    // entities.js, loaded once on the JS side. ---

    pub fn flipper_count(&self) -> u32 {
        self.world.flippers.len() as u32
    }
    pub fn flipper_angle(&self, i: u32) -> f64 {
        self.world.flippers[i as usize].angle
    }
    pub fn flipper_is_moving_up(&self, i: u32) -> bool {
        self.world.flippers[i as usize].is_moving_up
    }

    pub fn bumper_count(&self) -> u32 {
        self.world.bumpers.len() as u32
    }
    pub fn bumper_flash(&self, i: u32) -> i32 {
        self.world.bumpers[i as usize].flash
    }

    pub fn slingshot_count(&self) -> u32 {
        self.world.slingshots.len() as u32
    }
    pub fn slingshot_flash(&self, i: u32) -> i32 {
        self.world.slingshots[i as usize].flash
    }

    pub fn kickback_count(&self) -> u32 {
        self.world.kickbacks.len() as u32
    }
    pub fn kickback_flash(&self, i: u32) -> i32 {
        self.world.kickbacks[i as usize].flash
    }
}
