//! Port of `src/physics.js`.
//!
//! JS's collision functions are entangled with scoring, audio, and particle
//! side effects (`game.score += gain`, `playBumperHit()`, `spawnSpark(...)`)
//! because those modules (`state.js`, `audio.js`, `particles.js`,
//! `combo.js`) are plain mutable globals it can reach into directly. None of
//! those exist in Rust yet, so the scoring/effect-triggering collisions
//! (bumpers, slingshots) return an `Option<*Hit>` describing what happened
//! instead of causing the side effect themselves — the state.js port will
//! consume these and decide what to do (compute the combo-multiplied score,
//! play a sound, spawn a particle). Ball position/velocity and each
//! entity's own `flash` timer are intrinsic to the collision itself, so
//! those are still mutated in place here, matching JS.

use crate::ball::Ball;
use crate::constants::{
    BALL_R, FLIP_LEN, FLIP_R, GUIDE_R, Point, SLINGSHOT_FLASH_FRAMES, SLINGSHOT_KICK, SLINGSHOT_R,
};
use crate::entities::{Arc, Bumper, FlashSegment, Flipper, Post, Segment};

pub fn closest_point_on_segment(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> Point {
    let dx = bx - ax;
    let dy = by - ay;
    let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    Point { x: ax + t * dx, y: ay + t * dy }
}

/// Repositions `ball` to sit exactly `seg_r + BALL_R` from the segment when
/// it has penetrated it, then calls `on_hit(ball, nx, ny, dot)` with the
/// collision normal and the ball's velocity along it. Returns whether a
/// collision happened (JS has no equivalent return value — this is a
/// Rust-only testing hook, callers otherwise ignore it just like JS).
pub fn reflect_ball_off_segment(
    ball: &mut Ball,
    ax: f64,
    ay: f64,
    bx: f64,
    by: f64,
    seg_r: f64,
    mut on_hit: impl FnMut(&mut Ball, f64, f64, f64),
) -> bool {
    let cp = closest_point_on_segment(ball.x, ball.y, ax, ay, bx, by);
    let ex = ball.x - cp.x;
    let ey = ball.y - cp.y;
    let dist = ex.hypot(ey);
    let min = BALL_R + seg_r;
    if dist >= min || dist < 0.01 {
        return false;
    }
    let nx = ex / dist;
    let ny = ey / dist;
    ball.x = cp.x + nx * min;
    ball.y = cp.y + ny * min;
    let dot = ball.vx * nx + ball.vy * ny;
    on_hit(ball, nx, ny, dot);
    true
}

pub fn collide_flipper(ball: &mut Ball, f: &Flipper) {
    let tip_x = f.px + f.angle.cos() * FLIP_LEN;
    let tip_y = f.py + f.angle.sin() * FLIP_LEN;

    reflect_ball_off_segment(ball, f.px, f.py, tip_x, tip_y, FLIP_R, |b, nx, ny, dot| {
        // En "aktiv flick" kräver att flippern faktiskt är mitt i sin rörelse uppåt.
        if f.is_moving_up {
            // Aktiv flick: hela vektorn boostas rejält.
            b.vx = (b.vx - 2.0 * dot * nx) * 1.8;
            b.vy = (b.vy - 2.0 * dot * ny) * 1.8;
            if b.vy > -2.0 {
                b.vy -= 4.0;
            }
        } else {
            // Stillastående flipper: bara normalkomponenten dämpas så att
            // bollen kan rulla snyggt längs flippern.
            b.vx -= 1.4 * dot * nx;
            b.vy -= 1.4 * dot * ny;
        }
    });
}

/// What a bumper hit did to the ball, for the (not-yet-ported) state.js
/// caller to score/sound/spark — mirrors what `collideBumper` in JS reads
/// off `ball`/`b` after repositioning it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BumperHit {
    pub x: f64,
    pub y: f64,
    pub special: bool,
}

pub fn collide_bumper(ball: &mut Ball, b: &mut Bumper) -> Option<BumperHit> {
    let dx = ball.x - b.x;
    let dy = ball.y - b.y;
    let dist = dx.hypot(dy);
    let min = BALL_R + b.r;
    if dist >= min || dist < 0.01 {
        return None;
    }
    b.flash = 18;
    let nx = dx / dist;
    let ny = dy / dist;
    ball.x = b.x + nx * min;
    ball.y = b.y + ny * min;
    // 15% dämpning per studs — annars konserverar bumpern farten nästan
    // exakt och bollen kan studsa runt bumper-triangeln snabbt utan att
    // tappa fart.
    let speed = (ball.vx.hypot(ball.vy) * 0.85).max(6.0);
    ball.vx = nx * speed;
    ball.vy = ny * speed;
    Some(BumperHit { x: ball.x, y: ball.y, special: b.special })
}

// En stum stolpe (ingen poäng/ljud) som studsar bollen radiellt bort från
// sin mittpunkt, till skillnad från en guide som bara håller bollen på ena
// sidan av en linje.
pub fn collide_post(ball: &mut Ball, p: &Post) {
    let dx = ball.x - p.x;
    let dy = ball.y - p.y;
    let dist = dx.hypot(dy);
    let min = BALL_R + p.r;
    if dist >= min || dist < 0.01 {
        return;
    }
    let nx = dx / dist;
    let ny = dy / dist;
    ball.x = p.x + nx * min;
    ball.y = p.y + ny * min;
    let speed = ball.vx.hypot(ball.vy) * 0.9;
    ball.vx = nx * speed;
    ball.vy = ny * speed;
}

pub fn collide_guide(ball: &mut Ball, g: &Segment) {
    reflect_ball_off_segment(ball, g.x1, g.y1, g.x2, g.y2, GUIDE_R, |b, nx, ny, dot| {
        if dot < 0.0 {
            // Bara normalkomponenten studsar/dämpas (0.7 = restitution) —
            // tangentialfarten (rullningen längs guiden) lämnas orörd.
            b.vx -= 1.7 * dot * nx;
            b.vy -= 1.7 * dot * ny;
        }
    });
}

/// Bågen som bildar takets halvcirkel hanteras som en riktig cirkel istället
/// för sina rit-segment — annars kan bollen träffa två grannsegment med
/// olika normaler i samma bildruta och få en skevande stöt.
pub fn collide_arc(
    ball: &mut Ball,
    arc: &Arc,
    seg_r: f64,
    mut on_hit: impl FnMut(&mut Ball, f64, f64, f64),
) -> bool {
    let dx = ball.x - arc.center.x;
    let dy = ball.y - arc.center.y;
    if dy > 0.0 {
        return false; // bara övre halvan av bågen är i bruk
    }
    let dist = dx.hypot(dy);
    if dist < 0.01 {
        return false;
    }
    let diff = dist - arc.r; // positivt = utanför bågen, negativt = innanför
    let min = BALL_R + seg_r;
    if diff.abs() >= min {
        return false;
    }
    let sign = if diff >= 0.0 { 1.0 } else { -1.0 };
    let ux = dx / dist;
    let uy = dy / dist;
    let nx = sign * ux;
    let ny = sign * uy;
    ball.x = arc.center.x + ux * (arc.r + sign * min);
    ball.y = arc.center.y + uy * (arc.r + sign * min);
    let dot = ball.vx * nx + ball.vy * ny;
    on_hit(ball, nx, ny, dot);
    true
}

pub fn collide_lane_curve(ball: &mut Ball, arc: &Arc) {
    collide_arc(ball, arc, GUIDE_R, |b, nx, ny, dot| {
        if dot < 0.0 {
            b.vx -= 1.7 * dot * nx;
            b.vy -= 1.7 * dot * ny;
        }
    });
}

/// Mirrors what `collideSlingshot` reads after the kick, for the state.js
/// caller to score/sound/spark.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlingshotHit {
    pub x: f64,
    pub y: f64,
}

pub fn collide_slingshot(ball: &mut Ball, s: &mut FlashSegment) -> Option<SlingshotHit> {
    let (x1, y1, x2, y2) = (s.x1, s.y1, s.x2, s.y2);
    let mut hit = None;
    reflect_ball_off_segment(ball, x1, y1, x2, y2, SLINGSHOT_R, |b, nx, ny, dot| {
        if dot >= 0.0 {
            return;
        }
        b.vx -= 2.0 * dot * nx;
        b.vy -= 2.0 * dot * ny;
        let speed = b.vx.hypot(b.vy) + SLINGSHOT_KICK;
        let ang = b.vy.atan2(b.vx);
        b.vx = ang.cos() * speed;
        b.vy = ang.sin() * speed;
        hit = Some(SlingshotHit { x: b.x, y: b.y });
    });
    if hit.is_some() {
        s.flash = SLINGSHOT_FLASH_FRAMES;
    }
    hit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Dir;

    fn flipper(is_moving_up: bool) -> Flipper {
        Flipper {
            px: 0.0,
            py: 0.0,
            angle: 0.0,
            rest_angle: 0.0,
            active_angle: 0.0,
            dir: Dir::Left,
            is_moving_up,
        }
    }

    #[test]
    fn closest_point_clamps_to_segment_ends() {
        // Point far beyond segment's `b` end should clamp to `b` itself.
        let cp = closest_point_on_segment(100.0, 0.0, 0.0, 0.0, 10.0, 0.0);
        assert_eq!((cp.x, cp.y), (10.0, 0.0));
    }

    #[test]
    fn collide_guide_only_damps_approaching_balls() {
        // Guide lies on y=0; ball sits below it (y>0), so the collision
        // normal points down and away (0,1). Moving up toward the guide
        // (vy<0) is "approaching" (dot<0) and should be damped.
        let g = Segment { x1: -10.0, y1: 0.0, x2: 10.0, y2: 0.0 };
        let mut ball = Ball { x: 0.0, y: BALL_R + GUIDE_R - 1.0, vx: 0.0, vy: -5.0, ..Default::default() };
        collide_guide(&mut ball, &g);
        assert!(ball.vy != -5.0, "approaching ball should be damped, got vy={}", ball.vy);

        // Ball already moving away (vy>0, dot >= 0) is repositioned but not damped.
        let mut ball = Ball { x: 0.0, y: BALL_R + GUIDE_R - 1.0, vx: 0.0, vy: 5.0, ..Default::default() };
        collide_guide(&mut ball, &g);
        assert_eq!(ball.vy, 5.0);
    }

    #[test]
    fn collide_bumper_bounces_ball_outward_and_sets_flash() {
        let mut b = Bumper { x: 0.0, y: 0.0, r: 20.0, flash: 0, special: false };
        let mut ball = Ball { x: 25.0, y: 0.0, vx: -5.0, vy: 0.0, ..Default::default() };
        let hit = collide_bumper(&mut ball, &mut b).expect("should register a hit");
        assert!(!hit.special);
        assert_eq!(b.flash, 18);
        assert!(ball.x > 25.0, "ball should be pushed further from the bumper center");
        assert!(ball.vx > 0.0, "ball should now move away from the bumper");
    }

    #[test]
    fn collide_bumper_no_hit_when_far_away() {
        let mut b = Bumper { x: 0.0, y: 0.0, r: 20.0, flash: 0, special: false };
        let mut ball = Ball { x: 1000.0, y: 0.0, vx: 0.0, vy: 0.0, ..Default::default() };
        assert!(collide_bumper(&mut ball, &mut b).is_none());
        assert_eq!(b.flash, 0);
    }

    #[test]
    fn collide_slingshot_kicks_ball_and_sets_flash_only_on_real_hit() {
        // Segment lies on y=0; ball below it (y>0), so approaching (moving
        // up into it) is vy<0, same convention as the guide test above.
        let mut s = FlashSegment { x1: -10.0, y1: 0.0, x2: 10.0, y2: 0.0, flash: 0 };
        let mut ball = Ball { x: 0.0, y: BALL_R + SLINGSHOT_R - 1.0, vx: 0.0, vy: -5.0, ..Default::default() };
        let hit = collide_slingshot(&mut ball, &mut s).expect("approaching ball should kick");
        assert_eq!(s.flash, SLINGSHOT_FLASH_FRAMES);
        assert_eq!((hit.x, hit.y), (ball.x, ball.y));
        assert!(ball.vy > 0.0, "slingshot should kick the ball back away from it");

        // A ball already moving away is repositioned (reflect always
        // repositions) but must not re-trigger the kick/flash.
        s.flash = 0;
        let mut ball = Ball { x: 0.0, y: BALL_R + SLINGSHOT_R - 1.0, vx: 0.0, vy: 5.0, ..Default::default() };
        assert!(collide_slingshot(&mut ball, &mut s).is_none());
        assert_eq!(s.flash, 0);
    }

    #[test]
    fn collide_flipper_flick_boosts_velocity_more_than_resting_bounce() {
        let f_resting = flipper(false);
        let f_flicking = flipper(true);

        // Ball resting just above the flipper's pivot end, moving down into it.
        let mut resting_ball = Ball { x: 0.0, y: BALL_R + FLIP_R - 1.0, vx: 0.0, vy: 5.0, ..Default::default() };
        collide_flipper(&mut resting_ball, &f_resting);

        let mut flicked_ball = Ball { x: 0.0, y: BALL_R + FLIP_R - 1.0, vx: 0.0, vy: 5.0, ..Default::default() };
        collide_flipper(&mut flicked_ball, &f_flicking);

        assert!(
            flicked_ball.vy < resting_ball.vy,
            "an active flick should send the ball off faster (more negative vy) than a resting bounce: flicked={} resting={}",
            flicked_ball.vy,
            resting_ball.vy
        );
    }

    #[test]
    fn collide_lane_curve_pushes_ball_back_inside_arc() {
        let arc = Arc { center: Point { x: 0.0, y: 0.0 }, r: 100.0 };
        // Ball just outside the arc, in its upper half (negative y relative
        // to the center), moving down toward the center — i.e. approaching
        // the arc surface from outside — should be bounced back away.
        let mut ball = Ball { x: 0.0, y: -(100.0 + BALL_R + GUIDE_R - 1.0), vx: 0.0, vy: 3.0, ..Default::default() };
        collide_lane_curve(&mut ball, &arc);
        assert!(ball.vy < 0.0, "ball approaching the arc from outside should bounce back away from it");
    }
}
