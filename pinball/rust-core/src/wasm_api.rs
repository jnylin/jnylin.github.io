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
//!   worth it. Only the fields that actually change over time are exposed
//!   this way.
//! - **Static layout** (entity positions, radii, endpoints — everything
//!   `entities.rs` builds once and never mutates) is also JSON, but fetched
//!   *once* at startup via [`PinballApi::layout`], not per frame. This used
//!   to be a second, hand-copied source of truth in `entities.js` — real
//!   duplication risk, since nothing enforced the JS and Rust copies of a
//!   bumper's position stayed in sync. Routing it through Rust instead
//!   means `entities.rs` is the *only* place table layout is ever written
//!   down.
//!
//! Indices/counts are `u32` rather than `usize` — `usize`'s width isn't
//! fixed across targets, and wasm-bindgen wants a concrete numeric type.

use wasm_bindgen::prelude::*;

use crate::entities;
use crate::world::World;

/// Deliberately not just `#[derive(Serialize)]` on `entities::Flipper` etc.
/// — those carry collision-only fields (`dir`, `rest_angle`, `is_moving_up`,
/// ...) that have no business in a rendering contract. These are narrow,
/// purpose-built DTOs for exactly what `render.js` needs once at startup.
#[derive(serde::Serialize)]
struct FlipperLayout {
    px: f64,
    py: f64,
}

#[derive(serde::Serialize)]
struct BumperLayout {
    x: f64,
    y: f64,
    r: f64,
    special: bool,
}

#[derive(serde::Serialize)]
struct PostLayout {
    x: f64,
    y: f64,
    r: f64,
}

#[derive(serde::Serialize)]
struct SegmentLayout {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl From<&entities::Segment> for SegmentLayout {
    fn from(s: &entities::Segment) -> Self {
        SegmentLayout { x1: s.x1, y1: s.y1, x2: s.x2, y2: s.y2 }
    }
}

impl From<&entities::FlashSegment> for SegmentLayout {
    fn from(s: &entities::FlashSegment) -> Self {
        SegmentLayout { x1: s.x1, y1: s.y1, x2: s.x2, y2: s.y2 }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Layout {
    flippers: Vec<FlipperLayout>,
    bumpers: Vec<BumperLayout>,
    posts: Vec<PostLayout>,
    slingshots: Vec<SegmentLayout>,
    kickbacks: Vec<SegmentLayout>,
    guides: Vec<SegmentLayout>,
    lane_curve_segments: Vec<SegmentLayout>,
    lane_gate: SegmentLayout,
}

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

    /// The full static table layout as JSON — call once at startup, not
    /// per frame (see the module doc comment). `lane_curve_segments` is the
    /// only piece not already sitting on `World`; it's rendering-only
    /// (physics collides against the exact arc, `lane_curve_arc`, instead),
    /// so it's just rebuilt from `entities::lane_curve_segments()` here.
    pub fn layout(&self) -> String {
        let layout = Layout {
            flippers: self.world.flippers.iter().map(|f| FlipperLayout { px: f.px, py: f.py }).collect(),
            bumpers: self
                .world
                .bumpers
                .iter()
                .map(|b| BumperLayout { x: b.x, y: b.y, r: b.r, special: b.special })
                .collect(),
            posts: self.world.posts.iter().map(|p| PostLayout { x: p.x, y: p.y, r: p.r }).collect(),
            slingshots: self.world.slingshots.iter().map(SegmentLayout::from).collect(),
            kickbacks: self.world.kickbacks.iter().map(SegmentLayout::from).collect(),
            guides: self.world.guides.iter().map(SegmentLayout::from).collect(),
            lane_curve_segments: entities::lane_curve_segments().iter().map(SegmentLayout::from).collect(),
            lane_gate: SegmentLayout::from(&self.world.lane_gate),
        };
        serde_json::to_string(&layout).unwrap_or_else(|_| "{}".to_string())
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
    // radii, endpoints etc. never change after construction — see
    // `layout()` above for those. ---

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

#[cfg(test)]
mod tests {
    use super::*;

    /// This is the actual JS/WASM FFI contract for static layout: proves
    /// the JSON has the right shape (camelCase top-level keys, right
    /// counts) and stays in sync with what World actually holds.
    #[test]
    fn layout_json_matches_world_entity_counts() {
        let api = PinballApi::new(0.0);
        let json = api.layout();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["flippers"].as_array().unwrap().len(), api.flipper_count() as usize);
        assert_eq!(parsed["bumpers"].as_array().unwrap().len(), api.bumper_count() as usize);
        assert_eq!(parsed["slingshots"].as_array().unwrap().len(), api.slingshot_count() as usize);
        assert_eq!(parsed["kickbacks"].as_array().unwrap().len(), api.kickback_count() as usize);
        assert_eq!(parsed["laneCurveSegments"].as_array().unwrap().len(), 32);
        assert!(parsed["laneGate"]["x1"].is_number());
        assert_eq!(parsed["bumpers"][3]["special"], true);
    }
}
