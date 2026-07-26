import { W, H, WALL, LANE_DIVIDER_X, LANE_CURVE_R, LANE_TOP, laneCurveCenter } from './constants.js';

// --- Flipprar ---
// px=130 ger ett viloläge-gap på ~2.5 bollbredder mellan topparna (128 gav
// ~2.7; 135 gav ~1.9 — för snävt sedan bumper-triangeln blev ensam kvar;
// 120 gav ~3.6). Liten, försiktig åtstramning av 128 — god marginal kvar
// till 135-gränsen.
const FLIPPER_PX = 130;
export const flippers = [
    { px: FLIPPER_PX,     py: H - 85, angle: 0.15 * Math.PI, restAngle: 0.15 * Math.PI, activeAngle: -0.1 * Math.PI, dir: 'left'  },
    { px: W - FLIPPER_PX, py: H - 85, angle: 0.85 * Math.PI, restAngle: 0.85 * Math.PI, activeAngle:  1.1 * Math.PI, dir: 'right' },
];

// --- Bumpers ---
export const bumpers = [
    { x: 140, y: 200, r: 20, flash: 0 },
    { x: 280, y: 180, r: 20, flash: 0 },
    { x: 210, y: 280, r: 20, flash: 0 },
];

// --- Slingshots ---
// Små gummiband som knuffar iväg bollen med extra fart när den träffar dem.
// Placerade diagonalt strax ovanför respektive outlane-mynning (guiden som
// löper längs sidoväggen ner mot draget) för att fånga upp och studsa
// tillbaka bollar som annars gled rakt ner i outlanen utan motstånd.
export const slingshots = [
    { x1: 47,     y1: H - 234, x2: 80,     y2: H - 185, flash: 0 },
    { x1: W - 87, y1: H - 234, x2: W - 110, y2: H - 185, flash: 0 },
];

export function arcSegments(center, r, a0, a1, steps) {
    const pts = [];
    for (let i = 0; i <= steps; i++) {
        const a = a0 + (a1 - a0) * (i / steps);
        pts.push({ x: center.x + r * Math.cos(a), y: center.y + r * Math.sin(a) });
    }
    const segs = [];
    for (let i = 0; i < pts.length - 1; i++) {
        segs.push({ x1: pts[i].x, y1: pts[i].y, x2: pts[i+1].x, y2: pts[i+1].y });
    }
    return segs;
}

// --- Ledskenor / Guider ---
const OUTLANE_INSET = 28;

export const guides = [
    // Huvudguidernas start är kortad ~30px från väggkanten/skiljeväggen —
    // annars delar de startpunkt exakt med outlane-guiderna nedan, och
    // mellanrummet mellan dem växer då från noll (för smalt för bollen tills
    // det hunnit bre ut sig ordentligt). Vid y=H-150 har det redan blivit
    // minst en bollbredd (18px) på båda sidor.
    { x1: 53,  y1: H - 150, x2: FLIPPER_PX, y2: H - 90 },
    // Spegling av vänsterguiden ovan
    { x1: 340, y1: H - 150, x2: W - FLIPPER_PX, y2: H - 90 },

    // Utlopp (outlanes) — tajta kanaler tätt intill väggen (vänster) och
    // skjutbanans skiljevägg (höger, för att inte krocka med rampen — den
    // kan därför inte börja lika långt ut som vänstersidans väggkant gör).
    // Hamnar bollen här går den rakt förbi flippern istället för att fångas
    // upp av huvudguiden ovan.
    { x1: WALL,              y1: H - 180, x2: WALL + OUTLANE_INSET,              y2: H - 20 },
    { x1: LANE_DIVIDER_X,    y1: H - 180, x2: LANE_DIVIDER_X - OUTLANE_INSET,    y2: H - 20 },

    // Halvcirkelbåge över hela toppen
    ...arcSegments(laneCurveCenter, LANE_CURVE_R, 0, -Math.PI, 32),

    // Skiljevägg för skjutbanan
    { x1: LANE_DIVIDER_X, y1: LANE_TOP, x2: LANE_DIVIDER_X, y2: H },
];
