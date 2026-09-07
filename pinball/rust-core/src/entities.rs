//! Port of `src/entities.js`. Table layout (flippers, bumpers, guides, ...)
//! built once at startup, mirroring the JS module-level `export const`s.
//!
//! JS lets `main.js`/`physics.js` bolt extra fields onto these objects at
//! runtime (e.g. `flipper.isMovingUp`). Rust structs can't grow fields like
//! that, so those get added explicitly when the physics/state pass ports
//! the code that needs them — this pass only covers what entities.js itself
//! constructs.

use crate::constants::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
pub struct Flipper {
    pub px: f64,
    pub py: f64,
    pub angle: f64,
    pub rest_angle: f64,
    pub active_angle: f64,
    pub dir: Dir,
    /// Set by the (not-yet-ported) main.js flipper-update step: true only
    /// while the flipper is actively swinging up under player input.
    /// physics.js's collideFlipper reads it to decide between a "flick"
    /// (velocity boosted) and a resting bounce (normal component damped).
    pub is_moving_up: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Bumper {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub flash: i32,
    pub special: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Post {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

/// A line segment with no extra state — guides, the lane divider, the gate.
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Same shape as `Segment`, plus the `flash` counter JS bolts onto
/// slingshots and kickbacks (decremented each frame, reset on a hit).
#[derive(Clone, Copy, Debug)]
pub struct FlashSegment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub flash: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Arc {
    pub center: Point,
    pub r: f64,
}

/// Mittpunkten för planets symmetriska innehåll (bumpers, flipprar och
/// huvudguidernas flipperände) är inte canvasens mitt (W/2 = 210), utan
/// mitten av den faktiska spelytan mellan vänstervägg och skjutbanans
/// skiljevägg. Rampen äter bara utrymme från högersidan, så allt som
/// speglas kring W/2 istället hamnar omärkligt för långt åt höger.
const TABLE_CENTER_X: f64 = (WALL + LANE_DIVIDER_X) / 2.0; // 190

// Halva avståndet (80px) från TABLE_CENTER_X ger samma viloläge-gap som
// innan omcentreringen: ~2.5 bollbredder mellan topparna.
const FLIPPER_HALF_SPAN: f64 = 80.0;
const FLIPPER_PX: f64 = TABLE_CENTER_X - FLIPPER_HALF_SPAN; // 110

const OUTLANE_INSET: f64 = 28.0;
// Outlane-guidernas övre ände låg tidigare exakt vid väggen/skiljeväggen —
// en boll tätt mot väggen kunde fastna i klämman mellan väggen och
// guidens ändpunkt. Några pixlar extra luft löser det.
const OUTLANE_GAP: f64 = 30.0;

pub fn flippers() -> Vec<Flipper> {
    vec![
        Flipper {
            px: FLIPPER_PX,
            py: H - 85.0,
            angle: 0.15 * std::f64::consts::PI,
            rest_angle: 0.15 * std::f64::consts::PI,
            active_angle: -0.1 * std::f64::consts::PI,
            dir: Dir::Left,
            is_moving_up: false,
        },
        Flipper {
            px: TABLE_CENTER_X + FLIPPER_HALF_SPAN,
            py: H - 85.0,
            angle: 0.85 * std::f64::consts::PI,
            rest_angle: 0.85 * std::f64::consts::PI,
            active_angle: 1.1 * std::f64::consts::PI,
            dir: Dir::Right,
            is_moving_up: false,
        },
    ]
}

pub fn bumpers() -> Vec<Bumper> {
    vec![
        Bumper { x: 120.0, y: 200.0, r: 20.0, flash: 0, special: false },
        Bumper { x: 240.0, y: 170.0, r: 20.0, flash: 0, special: false },
        Bumper { x: 190.0, y: 280.0, r: 20.0, flash: 0, special: false },
        // Egen specialbumper, medvetet utanför triangeln — ger mer poäng
        // och är den enda med gnisteffekt.
        Bumper { x: 340.0, y: 230.0, r: 18.0, flash: 0, special: true },
    ]
}

pub fn posts() -> Vec<Post> {
    vec![Post { x: 40.0, y: 290.0, r: 12.0 }]
}

pub fn slingshots() -> Vec<FlashSegment> {
    vec![
        FlashSegment { x1: 47.0, y1: H - 234.0, x2: 70.0, y2: H - 185.0, flash: 0 },
        FlashSegment { x1: W - 87.0, y1: H - 234.0, x2: W - 110.0, y2: H - 185.0, flash: 0 },
    ]
}

pub fn kickbacks() -> Vec<FlashSegment> {
    vec![
        FlashSegment { x1: 40.0, y1: 543.0, x2: 115.0, y2: 620.0, flash: 0 },
        FlashSegment { x1: 340.0, y1: 543.0, x2: 265.0, y2: 620.0, flash: 0 },
    ]
}

pub fn outlane_guides() -> Vec<Segment> {
    vec![
        Segment {
            x1: WALL + OUTLANE_GAP,
            y1: H - 115.0,
            x2: WALL + OUTLANE_INSET,
            y2: H - 20.0,
        },
        Segment {
            x1: LANE_DIVIDER_X - OUTLANE_GAP,
            y1: H - 115.0,
            x2: LANE_DIVIDER_X - OUTLANE_INSET,
            y2: H - 20.0,
        },
    ]
}

pub fn guides() -> Vec<Segment> {
    let mut segs = vec![
        // Huvudguidernas start är kortad ~30px från väggkanten/skiljeväggen.
        Segment { x1: 45.0, y1: H - 150.0, x2: FLIPPER_PX, y2: H - 90.0 },
        // Spegling av vänsterguiden ovan.
        Segment {
            x1: 335.0,
            y1: H - 150.0,
            x2: TABLE_CENTER_X + FLIPPER_HALF_SPAN,
            y2: H - 90.0,
        },
    ];
    segs.extend(outlane_guides());
    // Skiljevägg för skjutbanan.
    segs.push(Segment { x1: LANE_DIVIDER_X, y1: LANE_TOP, x2: LANE_DIVIDER_X, y2: H });
    segs
}

/// Halvcirkelbåge över hela toppen, som korda-segment — enbart för
/// rendering. Kollisionen mot den körs som en enda exakt cirkel
/// (se `lane_curve_arc`), inte mot dessa segment.
pub fn arc_segments(center: Point, r: f64, a0: f64, a1: f64, steps: u32) -> Vec<Segment> {
    let pts: Vec<Point> = (0..=steps)
        .map(|i| {
            let a = a0 + (a1 - a0) * (i as f64 / steps as f64);
            Point { x: center.x + r * a.cos(), y: center.y + r * a.sin() }
        })
        .collect();
    pts.windows(2)
        .map(|w| Segment { x1: w[0].x, y1: w[0].y, x2: w[1].x, y2: w[1].y })
        .collect()
}

pub fn lane_curve_segments() -> Vec<Segment> {
    arc_segments(
        Point { x: LANE_CURVE_CENTER.x, y: LANE_CURVE_CENTER.y },
        LANE_CURVE_R,
        0.0,
        -std::f64::consts::PI,
        32,
    )
}

pub fn lane_curve_arc() -> Arc {
    Arc { center: Point { x: LANE_CURVE_CENTER.x, y: LANE_CURVE_CENTER.y }, r: LANE_CURVE_R }
}

/// Envägsgrind vid skjutbanans mynning: tätar öppningen där skjutbanan
/// möter planet ovanför skiljeväggen. Passeras fritt på väg ut, stängs
/// sedan bakom bollen (se physics/state-porten för beteendet).
pub fn lane_gate() -> Segment {
    Segment { x1: LANE_DIVIDER_X, y1: LANE_TOP, x2: W - WALL - 5.0, y2: LANE_TOP - 45.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_match_js() {
        assert_eq!(flippers().len(), 2);
        assert_eq!(bumpers().len(), 4);
        assert_eq!(posts().len(), 1);
        assert_eq!(slingshots().len(), 2);
        assert_eq!(kickbacks().len(), 2);
        assert_eq!(outlane_guides().len(), 2);
        assert_eq!(guides().len(), 5); // 2 huvudguider + 2 outlane + 1 skiljevägg
        assert_eq!(lane_curve_segments().len(), 32);
    }

    #[test]
    fn key_coordinates_match_js() {
        let f = flippers();
        assert_eq!(f[0].px, 110.0);
        assert_eq!(f[1].px, 270.0);

        let b = bumpers();
        assert!(b[3].special);
        assert!(!b[0].special);

        let gate = lane_gate();
        assert_eq!(gate.x1, LANE_DIVIDER_X);
        assert_eq!(gate.y1, LANE_TOP);
    }
}
