//! Port of `src/ballSave.js` — a self-contained countdown timer, started on
//! launch and consumed (successfully or not) on drain. See [`Combo`] for
//! why this is a struct `GameState` owns rather than a module singleton.
//!
//! [`Combo`]: crate::combo::Combo

use crate::constants::BALL_SAVE_FRAMES;

#[derive(Clone, Copy, Debug, Default)]
pub struct BallSave {
    frames_left: i32,
}

impl BallSave {
    pub fn start(&mut self) {
        self.frames_left = BALL_SAVE_FRAMES;
    }

    pub fn consume(&mut self) {
        self.frames_left = 0;
    }

    pub fn tick(&mut self) {
        if self.frames_left > 0 {
            self.frames_left -= 1;
        }
    }

    pub fn is_active(&self) -> bool {
        self.frames_left > 0
    }

    pub fn fraction(&self) -> f64 {
        self.frames_left as f64 / BALL_SAVE_FRAMES as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_until_ticked_out() {
        let mut bs = BallSave::default();
        assert!(!bs.is_active());
        bs.start();
        assert!(bs.is_active());
        for _ in 0..BALL_SAVE_FRAMES {
            bs.tick();
        }
        assert!(!bs.is_active());
    }

    #[test]
    fn consume_ends_it_immediately() {
        let mut bs = BallSave::default();
        bs.start();
        bs.consume();
        assert!(!bs.is_active());
    }
}
