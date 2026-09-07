//! Port of `src/combo.js`.
//!
//! JS keeps this as module-level mutable globals (`let multiplier`,
//! `let framesLeft`), meaning there's implicitly only ever one combo timer
//! for the one game running in a page. Rust has no equivalent "just a
//! global" default, so this is a plain struct `GameState` owns an instance
//! of — same single-instance behavior, just via ownership instead of a
//! module singleton.

use crate::constants::{COMBO_MAX, COMBO_STEP, COMBO_WINDOW_FRAMES};

#[derive(Clone, Copy, Debug)]
pub struct Combo {
    multiplier: f64,
    frames_left: i32,
}

impl Default for Combo {
    fn default() -> Self {
        Combo { multiplier: 1.0, frames_left: 0 }
    }
}

impl Combo {
    /// Varje registrerad träff inom fönstret höjer multiplikatorn ett steg.
    /// Går fönstret ut innan nästa träff börjar kedjan om från 1x.
    pub fn register_hit(&mut self) -> f64 {
        self.multiplier =
            if self.frames_left > 0 { (self.multiplier + COMBO_STEP).min(COMBO_MAX) } else { 1.0 };
        self.frames_left = COMBO_WINDOW_FRAMES;
        self.multiplier
    }

    pub fn tick(&mut self) {
        if self.frames_left > 0 {
            self.frames_left -= 1;
        }
    }

    pub fn reset(&mut self) {
        self.multiplier = 1.0;
        self.frames_left = 0;
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    pub fn fraction(&self) -> f64 {
        self.frames_left as f64 / COMBO_WINDOW_FRAMES as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chains_within_window_and_resets_after_it_expires() {
        let mut combo = Combo::default();
        assert_eq!(combo.register_hit(), 1.0);
        assert_eq!(combo.register_hit(), 1.0 + COMBO_STEP);
        assert_eq!(combo.register_hit(), 1.0 + 2.0 * COMBO_STEP);

        for _ in 0..COMBO_WINDOW_FRAMES {
            combo.tick();
        }
        // Window has fully expired — next hit starts the chain over at 1x.
        assert_eq!(combo.register_hit(), 1.0);
    }

    #[test]
    fn caps_at_combo_max() {
        let mut combo = Combo::default();
        for _ in 0..20 {
            combo.register_hit();
        }
        assert_eq!(combo.multiplier(), COMBO_MAX);
    }
}
