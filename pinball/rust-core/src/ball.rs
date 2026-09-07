//! The ball object, matching what `state.js`'s `spawnBall` constructs.
//! `main.js`'s per-frame update loop (drain/kickback/launch-return checks)
//! is still a later pass — the fields here exist because state.js's own
//! functions (`launchBall`, `relaunchBall`, `handleDrain`) read/write them.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ball {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub dying: bool,
    pub dying_timer: i32,
    pub waiting: bool,
    pub has_escaped: bool,
    pub outlane_grace: i32,
}
