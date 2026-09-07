//! Port of `src/state.js` — the game state machine (score, balls, ball
//! lifecycle, nudge/tilt, drain handling).
//!
//! JS's version is entangled with the DOM: it reads/writes
//! `document.getElementById(...)`, `localStorage`, and sets Swedish UI
//! message text directly (`msgEl.textContent = '...'`) from inside the
//! same functions that mutate game state. None of that belongs in this
//! crate — Rust core has no DOM, no storage, and text is a presentation
//! concern for the (not-yet-ported) main.js/render.js glue layer to own,
//! in whatever language it wants. So instead:
//!
//! - Functions that would have set a message return a typed event
//!   (`LaunchOutcome`, `NudgeEvent`, `DrainEvent`) describing what
//!   happened; the JS caller maps that to text/sound/localStorage.
//! - `high_score` is a plain field the caller seeds from `localStorage` at
//!   startup (via [`GameState::new`]) and persists itself when
//!   `DrainEvent::GameOver { is_new_high_score: true, .. }` comes back —
//!   this crate never touches storage.
//! - `Math.random()` calls become a `rand_unit: f64` parameter (expected
//!   in `[0, 1)`, same range) on the functions that used it, so this crate
//!   stays a pure, deterministic, dependency-free simulation — the caller
//!   already has `Math.random()` for free and there's no need to pull in
//!   `rand`/`getrandom` for wasm this early just to reinvent it.
//! - JS's `nudgeResetTimer` is a real `setTimeout(2000ms)` — a wall-clock
//!   timer that doesn't fit this crate's frame-driven model at all. Ported
//!   as a frame counter (`NUDGE_RESET_FRAMES`, ~2s @60fps) instead, ticked
//!   once per frame by [`GameState::tick_nudge_reset`] alongside the other
//!   timers — the main.js port will call it from the per-frame update loop
//!   the same way it calls `tickBallSave()`/`tickCombo()` today.
//! - `ballReturnedToPlunger()` in JS has no state to mutate at all (it only
//!   sets message text), so it isn't ported here — the main.js port will
//!   emit that event itself when its kickback/launch-return checks fire.

use crate::ball::Ball;
use crate::ball_save::BallSave;
use crate::combo::Combo;
use crate::constants::{BALL_SAVE_LAUNCH_POWER, LAUNCH_MAX_VY, PLUNGER_X, PLUNGER_Y};

const NUDGE_RESET_FRAMES: i32 = 120; // ~2s @60fps, was a 2000ms setTimeout in JS

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Shake {
    pub x: f64,
    pub y: f64,
}

// `#[serde(tag = "type")]` (internally tagged) needs every variant to
// serialize as a map, so a bare `Launched(f64)`/`Warning(i32)` tuple variant
// won't work — hence the named `{ power: f64 }`/`{ count: i32 }` fields
// below instead of JS's plain callback arguments.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "type")]
pub enum LaunchOutcome {
    /// A fresh ball was launched, costing one of `balls_left`. Carries the
    /// power (0..1) it was launched at, for the caller to pass to
    /// `playLaunch(pwr)`.
    Launched { power: f64 },
    /// The same ball was fired again after rolling back down the lane —
    /// doesn't cost a ball. Carries launch power, same as `Launched`.
    Relaunched { power: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "type")]
pub enum NudgeEvent {
    /// Caller should show a "Nudge (n/3)" warning.
    Warning { count: i32 },
    /// Third nudge — flippers disabled until the next ball.
    Tilted,
}

// Nested inside `world::StepEvent::Drained` at serialization time — that
// enum tags itself with `#[serde(tag = "event")]` instead of `"type"`
// specifically so the two tag keys don't collide once merged into one
// JSON object.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "type")]
pub enum DrainEvent {
    /// Ball-save was active: a new ball was auto-launched at
    /// `BALL_SAVE_LAUNCH_POWER`, no ball lost. Caller should `playBallSave()`.
    BallSaved,
    /// Balls remain; caller should show the idle message.
    BallsRemain,
    /// That was the last ball.
    GameOver { score: i64, is_new_high_score: bool },
}

/// The game state machine. Owns a [`BallSave`] and [`Combo`] timer
/// instance each — see their module docs for why these are owned structs
/// rather than JS-style module-level globals.
pub struct GameState {
    pub score: i64,
    pub balls_left: i32,
    pub over: bool,
    pub ball: Option<Ball>,
    pub launch_power: f64,
    pub shake: Shake,
    pub tilted: bool,
    pub nudge_count: i32,
    pub high_score: i64,
    nudge_reset_frames_left: i32,
    ball_save: BallSave,
    combo: Combo,
}

impl GameState {
    /// `high_score` should come from wherever the caller persists it
    /// (JS: `localStorage.getItem('pinball-hi')`).
    pub fn new(high_score: i64) -> Self {
        GameState {
            score: 0,
            balls_left: 3,
            over: false,
            ball: None,
            launch_power: 0.0,
            shake: Shake::default(),
            tilted: false,
            nudge_count: 0,
            high_score,
            nudge_reset_frames_left: 0,
            ball_save: BallSave::default(),
            combo: Combo::default(),
        }
    }

    fn spawn_ball(&mut self, vy: f64, rand_unit: f64) {
        self.ball = Some(Ball {
            x: PLUNGER_X,
            y: PLUNGER_Y,
            vx: (rand_unit - 0.5) * 2.0,
            vy,
            ..Default::default()
        });
    }

    fn launch_ball(&mut self, rand_unit: f64) -> f64 {
        self.balls_left -= 1;
        let pwr = self.launch_power;
        self.launch_power = 0.0;
        self.spawn_ball(-pwr * LAUNCH_MAX_VY, rand_unit);
        self.ball_save.start();
        pwr
    }

    /// Skjuter iväg samma boll igen efter att den rullat tillbaka ner i
    /// röret — kostar ingen ny boll eftersom den aldrig kom i spel.
    ///
    /// JS assumes `game.ball` exists (enforced by `triggerLaunch`'s guard)
    /// and would throw otherwise; this no-ops instead if called without a
    /// ball. [`Self::trigger_launch`] is the safe public entry point that
    /// upholds the same invariant JS relies on.
    fn relaunch_ball(&mut self, rand_unit: f64) -> f64 {
        let pwr = self.launch_power;
        self.launch_power = 0.0;
        if let Some(b) = self.ball.as_mut() {
            b.x = PLUNGER_X;
            b.y = PLUNGER_Y;
            b.vx = (rand_unit - 0.5) * 2.0;
            b.vy = -pwr * LAUNCH_MAX_VY;
            b.waiting = false;
            b.has_escaped = false;
        }
        self.ball_save.start();
        pwr
    }

    /// Called when Space is released (or the launch gesture ends). Decides
    /// between a fresh launch and a relaunch of a ball waiting in the lane,
    /// matching `triggerLaunch`'s branching in JS.
    pub fn trigger_launch(&mut self, rand_unit: f64) -> Option<LaunchOutcome> {
        if self.over || self.launch_power < 0.1 {
            return None;
        }
        if self.ball.is_none() && self.balls_left > 0 {
            Some(LaunchOutcome::Launched { power: self.launch_ball(rand_unit) })
        } else if self.ball.as_ref().is_some_and(|b| b.waiting) {
            Some(LaunchOutcome::Relaunched { power: self.relaunch_ball(rand_unit) })
        } else {
            None
        }
    }

    pub fn nudge_game(&mut self, force_x: f64, force_y: f64) -> Option<NudgeEvent> {
        if self.tilted || self.ball.is_none() || self.over {
            return None;
        }
        if let Some(b) = self.ball.as_mut() {
            b.vx += force_x;
            b.vy += force_y;
        }
        self.shake.x = force_x * 2.5;
        self.shake.y = force_y * 2.5;
        self.nudge_count += 1;
        self.nudge_reset_frames_left = NUDGE_RESET_FRAMES;

        if self.nudge_count >= 3 {
            self.tilted = true;
            Some(NudgeEvent::Tilted)
        } else {
            Some(NudgeEvent::Warning { count: self.nudge_count })
        }
    }

    /// Call once per frame. Returns `true` exactly on the frame the
    /// nudge-warning window expires *and* the caller should reset the
    /// message back to the idle flipper hint (JS's condition: not tilted,
    /// not game over, and a ball still in play).
    pub fn tick_nudge_reset(&mut self) -> bool {
        if self.nudge_reset_frames_left == 0 {
            return false;
        }
        self.nudge_reset_frames_left -= 1;
        if self.nudge_reset_frames_left > 0 {
            return false;
        }
        self.nudge_count = 0;
        !self.tilted && !self.over && self.ball.is_some()
    }

    pub fn tick_ball_save(&mut self) {
        self.ball_save.tick();
    }

    pub fn tick_combo(&mut self) {
        self.combo.tick();
    }

    pub fn is_ball_save_active(&self) -> bool {
        self.ball_save.is_active()
    }

    pub fn ball_save_fraction(&self) -> f64 {
        self.ball_save.fraction()
    }

    pub fn combo_multiplier(&self) -> f64 {
        self.combo.multiplier()
    }

    pub fn combo_fraction(&self) -> f64 {
        self.combo.fraction()
    }

    /// A bumper was hit (see `physics::collide_bumper`'s `BumperHit`).
    /// Applies the combo multiplier and adds to score. Returns the points
    /// gained, for the caller to pass to `spawnScorePopup`.
    pub fn score_bumper_hit(&mut self, special: bool) -> i64 {
        let mult = self.combo.register_hit();
        let gain = ((if special { 100.0 } else { 50.0 }) * mult).round() as i64;
        self.score += gain;
        gain
    }

    /// A slingshot was hit (see `physics::collide_slingshot`'s
    /// `SlingshotHit`). Same shape as [`Self::score_bumper_hit`].
    pub fn score_slingshot_hit(&mut self) -> i64 {
        let mult = self.combo.register_hit();
        let gain = (10.0 * mult).round() as i64;
        self.score += gain;
        gain
    }

    /// The ball fell out the bottom. JS calls `playDrain()` unconditionally
    /// at the top of `handleDrain` regardless of outcome — the caller
    /// should do the same before matching on the returned [`DrainEvent`].
    pub fn handle_drain(&mut self, rand_unit: f64) -> DrainEvent {
        let was_tilted = self.tilted;
        self.tilted = false;
        self.nudge_count = 0;

        if !was_tilted && self.ball_save.is_active() {
            self.ball_save.consume();
            self.spawn_ball(-BALL_SAVE_LAUNCH_POWER * LAUNCH_MAX_VY, rand_unit);
            return DrainEvent::BallSaved;
        }

        // En tilt förverkar ball-save.
        self.ball_save.consume();
        self.combo.reset();

        if self.balls_left == 0 {
            self.over = true;
            let is_new_high_score = self.score > self.high_score;
            if is_new_high_score {
                self.high_score = self.score;
            }
            DrainEvent::GameOver { score: self.score, is_new_high_score }
        } else {
            DrainEvent::BallsRemain
        }
    }

    pub fn reset(&mut self) {
        self.score = 0;
        self.balls_left = 3;
        self.over = false;
        self.ball = None;
        self.launch_power = 0.0;
        self.tilted = false;
        self.nudge_count = 0;
        self.nudge_reset_frames_left = 0;
        self.shake = Shake::default();
        self.ball_save.consume();
        self.combo.reset();
        // high_score deliberately untouched — it outlives individual games.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_costs_a_ball_and_spawns_it_at_the_plunger() {
        let mut g = GameState::new(0);
        g.launch_power = 0.5;
        let outcome = g.trigger_launch(0.5);
        assert_eq!(outcome, Some(LaunchOutcome::Launched { power: 0.5 }));
        assert_eq!(g.balls_left, 2);
        assert_eq!(g.launch_power, 0.0);
        let b = g.ball.expect("ball should be spawned");
        assert_eq!((b.x, b.y), (PLUNGER_X, PLUNGER_Y));
        assert_eq!(b.vy, -0.5 * LAUNCH_MAX_VY);
    }

    #[test]
    fn weak_charge_does_not_launch() {
        let mut g = GameState::new(0);
        g.launch_power = 0.05;
        assert_eq!(g.trigger_launch(0.5), None);
        assert!(g.ball.is_none());
        assert_eq!(g.balls_left, 3);
    }

    #[test]
    fn relaunch_does_not_cost_a_ball() {
        let mut g = GameState::new(0);
        g.launch_power = 0.5;
        g.trigger_launch(0.5);
        g.ball.as_mut().unwrap().waiting = true;

        g.launch_power = 0.8;
        let outcome = g.trigger_launch(0.5);
        assert_eq!(outcome, Some(LaunchOutcome::Relaunched { power: 0.8 }));
        assert_eq!(g.balls_left, 2, "relaunch must not consume another ball");
        assert!(!g.ball.unwrap().waiting);
    }

    #[test]
    fn nudge_counts_up_and_tilts_on_third() {
        let mut g = GameState::new(0);
        g.launch_power = 0.5;
        g.trigger_launch(0.5);

        assert_eq!(g.nudge_game(1.0, 0.0), Some(NudgeEvent::Warning { count: 1 }));
        assert_eq!(g.nudge_game(1.0, 0.0), Some(NudgeEvent::Warning { count: 2 }));
        assert_eq!(g.nudge_game(1.0, 0.0), Some(NudgeEvent::Tilted));
        assert!(g.tilted);
    }

    #[test]
    fn nudge_does_nothing_once_tilted_or_without_a_ball() {
        let mut g = GameState::new(0);
        assert_eq!(g.nudge_game(1.0, 0.0), None, "no ball in play yet");

        g.launch_power = 0.5;
        g.trigger_launch(0.5);
        g.tilted = true;
        assert_eq!(g.nudge_game(1.0, 0.0), None, "already tilted");
    }

    #[test]
    fn nudge_reset_clears_count_after_window_and_reports_message_reset() {
        let mut g = GameState::new(0);
        g.launch_power = 0.5;
        g.trigger_launch(0.5);
        g.nudge_game(1.0, 0.0);

        for _ in 0..NUDGE_RESET_FRAMES - 1 {
            assert!(!g.tick_nudge_reset());
        }
        assert!(g.tick_nudge_reset(), "final tick should fire the reset");
        assert_eq!(g.nudge_count, 0);
    }

    #[test]
    fn ball_save_rescues_drain_without_losing_a_ball() {
        let mut g = GameState::new(0);
        g.launch_power = 0.5;
        g.trigger_launch(0.5); // starts ball-save

        let event = g.handle_drain(0.5);
        assert_eq!(event, DrainEvent::BallSaved);
        assert_eq!(g.balls_left, 2, "ball-save must not cost an extra ball");
        assert!(g.ball.is_some(), "a new ball should be auto-launched");
        assert!(!g.is_ball_save_active(), "ball-save is consumed by the rescue");
    }

    #[test]
    fn tilt_forfeits_ball_save() {
        let mut g = GameState::new(0);
        g.launch_power = 0.5;
        g.trigger_launch(0.5); // starts ball-save
        g.tilted = true;

        let event = g.handle_drain(0.5);
        assert_eq!(event, DrainEvent::BallsRemain);
        assert_eq!(g.balls_left, 2);
    }

    #[test]
    fn drain_on_last_ball_ends_game_and_reports_new_high_score() {
        let mut g = GameState::new(100);
        g.balls_left = 1;
        g.launch_power = 0.5;
        g.trigger_launch(0.5); // balls_left -> 0
        for _ in 0..crate::constants::BALL_SAVE_FRAMES {
            g.tick_ball_save(); // let ball-save expire so this drain really ends the game
        }
        g.score = 250;

        let event = g.handle_drain(0.5);
        assert_eq!(event, DrainEvent::GameOver { score: 250, is_new_high_score: true });
        assert!(g.over);
        assert_eq!(g.high_score, 250);
    }

    #[test]
    fn drain_does_not_report_new_high_score_when_lower() {
        let mut g = GameState::new(999);
        g.balls_left = 1;
        g.launch_power = 0.5;
        g.trigger_launch(0.5);
        for _ in 0..crate::constants::BALL_SAVE_FRAMES {
            g.tick_ball_save();
        }
        g.score = 10;

        let event = g.handle_drain(0.5);
        assert_eq!(event, DrainEvent::GameOver { score: 10, is_new_high_score: false });
        assert_eq!(g.high_score, 999);
    }

    #[test]
    fn bumper_score_uses_combo_multiplier_and_special_bonus() {
        let mut g = GameState::new(0);
        assert_eq!(g.score_bumper_hit(false), 50);
        assert_eq!(g.score, 50);
        // Second hit within the combo window steps the multiplier to 1.5x.
        assert_eq!(g.score_bumper_hit(true), 150); // 100 * 1.5
        assert_eq!(g.score, 200);
    }

    #[test]
    fn reset_clears_run_state_but_keeps_high_score() {
        let mut g = GameState::new(500);
        g.score = 999;
        g.balls_left = 0;
        g.over = true;
        g.tilted = true;
        g.reset();

        assert_eq!(g.score, 0);
        assert_eq!(g.balls_left, 3);
        assert!(!g.over);
        assert!(!g.tilted);
        assert!(g.ball.is_none());
        assert_eq!(g.high_score, 500);
    }
}
