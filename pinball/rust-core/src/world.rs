//! Port of `src/main.js`'s per-frame simulation: `updateFlippers`,
//! `decayShake`, `checkKickback`, `checkLaunchReturn`, and the main
//! `update(dtFactor)` loop. `World` bundles a [`GameState`] with the
//! entity lists `entities.js` builds, since main.js's loop needs both
//! together (colliding the ball against bumpers/guides/etc. *and*
//! scoring/tilting/draining as a result).
//!
//! What's deliberately different from JS, beyond the events-not-side-effects
//! pattern already used in `physics.rs`/`state.rs`:
//!
//! - **`tickParticles()` is not called here.** `particles.js` isn't ported;
//!   that stays entirely JS's concern for now.
//! - **Input is two plain booleans**, not `input.js`'s `leftDown()`/
//!   `rightDown()`. Keyboard/touch listeners are a DOM concern; the caller
//!   passes in what's currently pressed each frame.
//! - **The nudge-reset timer** (a JS `setTimeout`, see `state.rs`'s doc
//!   comment) is ticked once per [`World::step`] call, unconditionally,
//!   mirroring that the real timer counted down in wall-clock time
//!   regardless of what else was happening in the game.
//! - **`ballReturnedToPlunger()`** has no state of its own in JS (message
//!   text only) — `check_kickback`/`check_launch_return` report it as
//!   `StepEvent::ReturnedToPlunger` for the caller to turn into text.
//!
//! One bug fixed, not preserved: JS's `update(dtFactor)` calls
//! `updateFlippers()` with no argument, so it silently runs with the
//! function's default `dtFactor = 1` — flipper speed doesn't scale with
//! real elapsed time the way the ball's physics does, so a flipper swings
//! at a different effective speed at 120Hz than at 60Hz. That's a genuine
//! bug rather than an intentional design choice, so [`World::update_flippers`]
//! takes `dt_factor` and actually uses it; `src/main.js` got the matching
//! one-line fix (`updateFlippers(dtFactor)`).

use crate::ball::Ball;
use crate::constants::{
    BALL_R, DYING_FRAMES, FLIP_SPEED, G, GUIDE_R, H, KICKBACK_CATCH_R, LANE_DIVIDER_X,
    LAUNCH_MAX_VY, LAUNCH_RATE, MAX_SPEED, OUTLANE_GRACE_FRAMES, PLUNGER_X, PLUNGER_Y,
    SUBSTEP_DIST, W, WALL,
};
use crate::entities::{self, Arc, Bumper, Dir, FlashSegment, Flipper, Post, Segment};
use crate::physics::{
    closest_point_on_segment, collide_bumper, collide_flipper, collide_guide, collide_lane_curve,
    collide_post, collide_slingshot,
};
use crate::state::{DrainEvent, GameState, LaunchOutcome, NudgeEvent};

/// Tagged with `"event"` rather than `"type"` (`DrainEvent`'s own tag)
/// specifically so serializing `Drained(DrainEvent::GameOver { .. })` — an
/// internally-tagged enum nested inside another — doesn't produce two
/// competing `"type"` keys in the same JSON object.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "event")]
pub enum StepEvent {
    BumperHit { x: f64, y: f64, special: bool, gain: i64 },
    SlingshotHit { x: f64, y: f64, gain: i64 },
    /// A ball that rolled back down the lane, or into a kickback channel,
    /// was returned to the plunger — not a drain, no ball lost.
    ReturnedToPlunger,
    /// The nudge-warning window expired; caller should reset its message
    /// (see the doc comment above about this being a ported `setTimeout`).
    NudgeReset,
    Drained(DrainEvent),
}

pub struct World {
    pub state: GameState,
    pub flippers: Vec<Flipper>,
    pub bumpers: Vec<Bumper>,
    pub posts: Vec<Post>,
    pub slingshots: Vec<FlashSegment>,
    pub kickbacks: Vec<FlashSegment>,
    pub guides: Vec<Segment>,
    pub outlane_guides: Vec<Segment>,
    pub lane_gate: Segment,
    pub lane_curve_arc: Arc,
}

impl World {
    pub fn new(high_score: i64) -> Self {
        World {
            state: GameState::new(high_score),
            flippers: entities::flippers(),
            bumpers: entities::bumpers(),
            posts: entities::posts(),
            slingshots: entities::slingshots(),
            kickbacks: entities::kickbacks(),
            guides: entities::guides(),
            outlane_guides: entities::outlane_guides(),
            lane_gate: entities::lane_gate(),
            lane_curve_arc: entities::lane_curve_arc(),
        }
    }

    pub fn trigger_launch(&mut self, rand_unit: f64) -> Option<LaunchOutcome> {
        self.state.trigger_launch(rand_unit)
    }

    pub fn nudge(&mut self, force_x: f64, force_y: f64) -> Option<NudgeEvent> {
        self.state.nudge_game(force_x, force_y)
    }

    pub fn reset(&mut self) {
        self.state.reset();
    }

    fn update_flippers(&mut self, dt_factor: f64, left_pressing: bool, right_pressing: bool) {
        let tilted = self.state.tilted;
        for f in self.flippers.iter_mut() {
            let pressing = !tilted
                && ((f.dir == Dir::Left && left_pressing) || (f.dir == Dir::Right && right_pressing));
            let prev_angle = f.angle;

            if pressing {
                f.angle += FLIP_SPEED * dt_factor * if f.dir == Dir::Left { -1.0 } else { 1.0 };
                f.angle = if f.dir == Dir::Left {
                    f.angle.max(f.active_angle)
                } else {
                    f.angle.min(f.active_angle)
                };
            } else {
                f.angle += FLIP_SPEED * 0.6 * dt_factor * if f.dir == Dir::Left { 1.0 } else { -1.0 };
                f.angle = if f.dir == Dir::Left {
                    f.angle.min(f.rest_angle)
                } else {
                    f.angle.max(f.rest_angle)
                };
            }
            f.is_moving_up = pressing && (f.angle != prev_angle);
        }
    }

    fn decay_shake(&mut self) {
        self.state.shake.x *= 0.8;
        self.state.shake.y *= 0.8;
        if self.state.shake.x.abs() < 0.05 {
            self.state.shake.x = 0.0;
        }
        if self.state.shake.y.abs() < 0.05 {
            self.state.shake.y = 0.0;
        }
    }

    /// Fångar en boll som hittat in i kickback-rännan och nått dess bortre
    /// ände — men bara om bollen faktiskt snuddat outlanen nyligen (se
    /// `Ball::outlane_grace`). Räknas inte som drain.
    fn check_kickback(&mut self, ball: &mut Ball) -> bool {
        if ball.outlane_grace <= 0 {
            return false;
        }
        for k in self.kickbacks.iter_mut() {
            if (ball.x - k.x2).hypot(ball.y - k.y2) < KICKBACK_CATCH_R {
                k.flash = 12;
                ball.x = PLUNGER_X;
                ball.y = PLUNGER_Y;
                ball.vx = 0.0;
                ball.vy = 0.0;
                ball.waiting = true;
                ball.outlane_grace = 0;
                return true;
            }
        }
        false
    }

    /// Runs one frame of simulation. `dt_factor` is elapsed-time-relative-
    /// to-60fps (see `main.js`'s `loop`, not yet ported — a real caller
    /// computes this from `requestAnimationFrame` timestamps). `rand_unit`
    /// is consumed only if a ball-save auto-relaunch happens to fall on
    /// this frame's drain (see `state.rs` for why randomness is a
    /// parameter instead of a dependency).
    pub fn step(
        &mut self,
        dt_factor: f64,
        left_pressing: bool,
        right_pressing: bool,
        charging: bool,
        rand_unit: f64,
    ) -> Vec<StepEvent> {
        let mut events = Vec::new();

        self.update_flippers(dt_factor, left_pressing, right_pressing);
        self.decay_shake();
        if self.state.tick_nudge_reset() {
            events.push(StepEvent::NudgeReset);
        }

        let ball_active = self.state.ball.as_ref().is_some_and(|b| !b.waiting);

        if !ball_active && !self.state.over && charging {
            self.state.launch_power = (self.state.launch_power + LAUNCH_RATE * dt_factor).min(1.0);
        }

        if !ball_active {
            return events;
        }

        self.state.tick_ball_save();
        self.state.tick_combo();

        // Take the ball out of GameState for the rest of this frame: physics
        // needs `&mut Ball` alongside `&mut self.state` for scoring, which
        // Rust can't borrow simultaneously while `ball` lives inside
        // `self.state.ball`. Put back at the end (or left out, on a drain).
        let mut ball = self.state.ball.take().unwrap();

        if ball.dying {
            ball.dying_timer -= 1;
            if ball.dying_timer <= 0 {
                events.push(StepEvent::Drained(self.state.handle_drain(rand_unit)));
            } else {
                self.state.ball = Some(ball);
            }
            return events;
        }

        ball.vy += G * dt_factor;
        ball.vx = ball.vx.clamp(-MAX_SPEED, MAX_SPEED);
        let vy_cap = if ball.x > LANE_DIVIDER_X { LAUNCH_MAX_VY } else { MAX_SPEED };
        ball.vy = ball.vy.clamp(-vy_cap, vy_cap);

        let steps = (ball.vx.abs().max(ball.vy.abs()) / SUBSTEP_DIST).ceil().max(1.0) as u32;
        for _ in 0..steps {
            ball.x += (ball.vx * dt_factor) / steps as f64;
            ball.y += (ball.vy * dt_factor) / steps as f64;

            if ball.x - BALL_R < WALL {
                ball.vx = ball.vx.abs() * 0.75;
                ball.x = WALL + BALL_R;
            }
            if ball.x + BALL_R > W - WALL {
                ball.vx = -ball.vx.abs() * 0.75;
                ball.x = W - WALL - BALL_R;
            }

            // Grinden stängs först nästa substep efter att bollen lämnat
            // banan, så den inte kan råka stänga på sig själv i samma steg
            // den passerar.
            let gate_active = ball.has_escaped;
            if !ball.has_escaped && ball.x < LANE_DIVIDER_X {
                ball.has_escaped = true;
            }
            if gate_active {
                collide_guide(&mut ball, &self.lane_gate);
            }

            for bmp in self.bumpers.iter_mut() {
                if let Some(hit) = collide_bumper(&mut ball, bmp) {
                    let gain = self.state.score_bumper_hit(hit.special);
                    events.push(StepEvent::BumperHit { x: hit.x, y: hit.y, special: hit.special, gain });
                }
            }
            for p in self.posts.iter() {
                collide_post(&mut ball, p);
            }
            for sl in self.slingshots.iter_mut() {
                if let Some(hit) = collide_slingshot(&mut ball, sl) {
                    let gain = self.state.score_slingshot_hit();
                    events.push(StepEvent::SlingshotHit { x: hit.x, y: hit.y, gain });
                }
            }
            for k in self.kickbacks.iter() {
                collide_guide(&mut ball, &Segment { x1: k.x1, y1: k.y1, x2: k.x2, y2: k.y2 });
            }
            for g in self.guides.iter() {
                collide_guide(&mut ball, g);
            }
            collide_lane_curve(&mut ball, &self.lane_curve_arc);
            for f in self.flippers.iter() {
                collide_flipper(&mut ball, f);
            }

            for og in self.outlane_guides.iter() {
                if touches_segment(&ball, og, 1.0) {
                    ball.outlane_grace = OUTLANE_GRACE_FRAMES;
                }
            }
        }

        for bmp in self.bumpers.iter_mut() {
            if bmp.flash > 0 {
                bmp.flash -= 1;
            }
        }
        for sl in self.slingshots.iter_mut() {
            if sl.flash > 0 {
                sl.flash -= 1;
            }
        }
        for k in self.kickbacks.iter_mut() {
            if k.flash > 0 {
                k.flash -= 1;
            }
        }
        if ball.outlane_grace > 0 {
            ball.outlane_grace -= 1;
        }

        if self.check_kickback(&mut ball) {
            events.push(StepEvent::ReturnedToPlunger);
            self.state.ball = Some(ball);
            return events;
        }
        if check_launch_return(&mut ball) {
            events.push(StepEvent::ReturnedToPlunger);
            self.state.ball = Some(ball);
            return events;
        }

        if ball.y > H + 20.0 {
            ball.dying = true;
            ball.dying_timer = DYING_FRAMES;
        }

        self.state.ball = Some(ball);
        events
    }
}

fn touches_segment(ball: &Ball, seg: &Segment, extra: f64) -> bool {
    let cp = closest_point_on_segment(ball.x, ball.y, seg.x1, seg.y1, seg.x2, seg.y2);
    (ball.x - cp.x).hypot(ball.y - cp.y) < BALL_R + GUIDE_R + extra
}

/// Fångar en boll som inte orkade hela vägen upp genom skjutbanan och föll
/// tillbaka, istället för att låta den falla vidare ut genom botten (drain).
fn check_launch_return(ball: &mut Ball) -> bool {
    let in_lane = ball.x > LANE_DIVIDER_X;
    if in_lane && ball.y >= PLUNGER_Y && ball.vy >= 0.0 {
        ball.x = PLUNGER_X;
        ball.y = PLUNGER_Y;
        ball.vx = 0.0;
        ball.vy = 0.0;
        ball.waiting = true;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launched_world() -> World {
        let mut w = World::new(0);
        w.state.launch_power = 0.5;
        w.trigger_launch(0.5);
        w
    }

    /// This is the actual JS/WASM FFI contract: proves the nested internally
    /// tagged enums (`StepEvent`'s `"event"` tag, `DrainEvent`'s `"type"`
    /// tag) serialize to two distinct keys instead of one clobbering the
    /// other, and spot-checks a plain event's shape too.
    #[test]
    fn step_events_serialize_to_the_expected_json_shape() {
        let drained = StepEvent::Drained(DrainEvent::GameOver { score: 250, is_new_high_score: true });
        let json = serde_json::to_string(&drained).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["event"], "Drained");
        assert_eq!(parsed["type"], "GameOver");
        assert_eq!(parsed["score"], 250);
        assert_eq!(parsed["is_new_high_score"], true);

        let hit = StepEvent::BumperHit { x: 1.5, y: 2.5, special: true, gain: 100 };
        let json = serde_json::to_string(&hit).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["event"], "BumperHit");
        assert_eq!(parsed["x"], 1.5);
        assert_eq!(parsed["special"], true);
    }

    #[test]
    fn charges_launch_power_only_while_ball_absent_or_waiting() {
        let mut w = World::new(0);
        w.step(1.0, false, false, true, 0.5);
        assert!(w.state.launch_power > 0.0);

        w.state.launch_power = 0.5;
        w.trigger_launch(0.5); // spawns an active ball
        let before = w.state.launch_power;
        w.step(1.0, false, false, true, 0.5);
        assert_eq!(w.state.launch_power, before, "charging must stop once a ball is in play");
    }

    #[test]
    fn launch_power_caps_at_one() {
        let mut w = World::new(0);
        for _ in 0..1000 {
            w.step(1.0, false, false, true, 0.5);
        }
        assert_eq!(w.state.launch_power, 1.0);
    }

    #[test]
    fn wall_bounce_reverses_and_damps_vx() {
        let mut w = launched_world();
        {
            let b = w.state.ball.as_mut().unwrap();
            b.x = WALL + BALL_R + 0.5;
            b.y = 500.0; // clear of every other entity — isolates the wall bounce
            b.vx = -10.0;
            b.vy = 0.0;
        }
        w.step(1.0, false, false, false, 0.5);
        let b = w.state.ball.unwrap();
        assert!(b.vx > 0.0, "ball should bounce off the left wall");
        assert!(b.vx < 10.0, "wall bounce should damp speed (0.75x)");
    }

    #[test]
    fn flippers_return_to_rest_when_tilted_even_if_pressed() {
        let mut w = launched_world();
        w.state.tilted = true;
        let rest_before = w.flippers[0].rest_angle;
        w.flippers[0].angle = w.flippers[0].active_angle;
        w.step(1.0, true, true, false, 0.5);
        // Not pressing (tilted overrides input) => angle moves toward rest.
        assert_ne!(w.flippers[0].angle, w.flippers[0].active_angle);
        let _ = rest_before;
    }

    #[test]
    fn flipper_speed_scales_with_dt_factor() {
        // Regression test for the fixed bug: updateFlippers must actually
        // use the frame's dt_factor, not silently run at a fixed rate.
        let mut slow = launched_world();
        let mut fast = launched_world();
        slow.step(0.5, true, false, false, 0.5);
        fast.step(2.0, true, false, false, 0.5);

        // flippers[0] is the left flipper (see entities::flippers()).
        let moved_slow = (slow.flippers[0].angle - slow.flippers[0].rest_angle).abs();
        let moved_fast = (fast.flippers[0].angle - fast.flippers[0].rest_angle).abs();
        assert!(
            moved_fast > moved_slow,
            "a bigger dt_factor should swing the flipper further this frame: fast={} slow={}",
            moved_fast,
            moved_slow
        );
    }

    #[test]
    fn ball_dies_after_falling_below_bottom_and_eventually_drains() {
        let mut w = launched_world();
        // Exhaust ball-save first so the drain below is a real one — otherwise
        // it gets rescued (StepEvent::Drained(DrainEvent::BallSaved)) and a
        // fresh ball is auto-relaunched instead of `ball` staying `None`.
        for _ in 0..crate::constants::BALL_SAVE_FRAMES {
            w.state.tick_ball_save();
        }
        {
            let b = w.state.ball.as_mut().unwrap();
            b.x = 200.0;
            b.y = H + 21.0;
            b.vx = 0.0;
            b.vy = 0.0;
        }
        w.step(1.0, false, false, false, 0.5);
        assert!(w.state.ball.as_ref().unwrap().dying);

        let mut drained = false;
        for _ in 0..(DYING_FRAMES + 1) {
            let events = w.step(1.0, false, false, false, 0.5);
            if events.iter().any(|e| matches!(e, StepEvent::Drained(_))) {
                drained = true;
                break;
            }
        }
        assert!(drained, "ball should eventually drain once dyingTimer runs out");
        assert!(w.state.ball.is_none());
    }

    #[test]
    fn check_launch_return_sends_ball_back_to_plunger() {
        let mut ball = Ball {
            x: PLUNGER_X,
            y: PLUNGER_Y + 5.0,
            vx: 3.0,
            vy: 1.0,
            ..Default::default()
        };
        assert!(check_launch_return(&mut ball));
        assert_eq!((ball.x, ball.y), (PLUNGER_X, PLUNGER_Y));
        assert!(ball.waiting);
    }

    #[test]
    fn check_launch_return_ignores_ball_still_rising() {
        let mut ball = Ball {
            x: PLUNGER_X,
            y: PLUNGER_Y + 5.0,
            vx: 0.0,
            vy: -3.0, // still moving up the lane
            ..Default::default()
        };
        assert!(!check_launch_return(&mut ball));
    }

    #[test]
    fn check_kickback_requires_recent_outlane_grace() {
        let mut w = World::new(0);
        let k = w.kickbacks[0];
        let mut ball = Ball { x: k.x2, y: k.y2, outlane_grace: 0, ..Default::default() };
        assert!(!w.check_kickback(&mut ball), "no recent outlane touch => no rescue");

        ball.outlane_grace = 5;
        assert!(w.check_kickback(&mut ball));
        assert!(ball.waiting);
        assert_eq!((ball.x, ball.y), (PLUNGER_X, PLUNGER_Y));
    }

    #[test]
    fn nudge_reset_event_fires_once_after_the_window() {
        let mut w = launched_world();
        w.nudge(1.0, 0.0);
        let mut fired = 0;
        for _ in 0..200 {
            let events = w.step(1.0, false, false, false, 0.5);
            fired += events.iter().filter(|e| matches!(e, StepEvent::NudgeReset)).count();
        }
        assert_eq!(fired, 1);
    }
}
