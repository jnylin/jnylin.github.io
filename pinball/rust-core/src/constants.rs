//! Direct port of `src/constants.js`. Keep values and names in sync with the
//! JS original — this crate is meant to replace that file's logic, not
//! diverge from it.

pub const W: f64 = 420.0;
pub const H: f64 = 660.0;
pub const G: f64 = 0.3;
pub const MAX_SPEED: f64 = 20.0;
pub const BALL_R: f64 = 9.0;
pub const WALL: f64 = 16.0;
pub const FLIP_LEN: f64 = 65.0;
pub const FLIP_R: f64 = 7.0;
pub const FLIP_SPEED: f64 = 0.18;
pub const GUIDE_R: f64 = 4.0;
pub const SUBSTEP_DIST: f64 = 6.0;
pub const LANE_W: f64 = 40.0;
pub const LANE_DIVIDER_X: f64 = W - WALL - LANE_W;

// Taket som en halvcirkel över hela spelfältets bredd.
pub const PLAYFIELD_W: f64 = W - 2.0 * WALL;
pub const LANE_CURVE_R: f64 = PLAYFIELD_W / 2.0;
pub const LANE_TOP: f64 = LANE_CURVE_R;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub const LANE_CURVE_CENTER: Point = Point { x: W / 2.0, y: LANE_TOP };

pub const PLUNGER_X: f64 = (LANE_DIVIDER_X + (W - WALL)) / 2.0;
pub const PLUNGER_Y: f64 = H - 60.0;
// Egen hastighetsgräns för utskjutning, frikopplad från MAX_SPEED — annars
// klipps kraften ner till det generella taket innan bollen hunnit ta sig
// upp genom skjutbanan (som kräver ~15.6 i starthastighet för att nå toppen).
pub const LAUNCH_MAX_VY: f64 = 32.0;
pub const LAUNCH_RATE: f64 = 1.0 / 70.0;
pub const DYING_FRAMES: i32 = 25;

// --- Slingshots ---
pub const SLINGSHOT_R: f64 = 6.0;
pub const SLINGSHOT_KICK: f64 = 6.0;
pub const SLINGSHOT_FLASH_FRAMES: i32 = 12;

// --- Ball-save ---
pub const BALL_SAVE_FRAMES: i32 = 180; // ~3s @60fps
pub const BALL_SAVE_LAUNCH_POWER: f64 = 0.75;

// --- Combo-multiplikator ---
pub const COMBO_WINDOW_FRAMES: i32 = 90; // ~1.5s @60fps mellan träffar
pub const COMBO_STEP: f64 = 0.5;
pub const COMBO_MAX: f64 = 4.0;

// --- main.js-lokala konstanter (flyttade hit eftersom entities.js/physics.js
// inte är rätt plats för dem, men de är genuina fysik-/timingkonstanter) ---
pub const KICKBACK_CATCH_R: f64 = 20.0;
// Hur länge (i bildrutor) en snuddning på outlanen "räknas" innan kickback-
// rännan får utlösa en räddning, se world.rs::check_kickback.
pub const OUTLANE_GRACE_FRAMES: i32 = 20;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_js_derived_values() {
        assert_eq!(LANE_DIVIDER_X, 364.0);
        assert_eq!(PLAYFIELD_W, 388.0);
        assert_eq!(LANE_CURVE_R, 194.0);
        assert_eq!(PLUNGER_X, 384.0);
        assert_eq!(PLUNGER_Y, 600.0);
    }
}
