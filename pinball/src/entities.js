import { W, H, WALL, LANE_DIVIDER_X, LANE_CURVE_R, LANE_TOP, laneCurveCenter } from './constants.js';

// Mittpunkten för planets symmetriska innehåll (bumpers, flipprar och
// huvudguidernas flipperände) är inte canvasens mitt (W/2 = 210), utan
// mitten av den faktiska spelytan mellan vänstervägg och skjutbanans
// skiljevägg. Rampen äter bara utrymme från högersidan, så allt som
// speglas kring W/2 istället hamnar omärkligt för långt åt höger.
const TABLE_CENTER_X = (WALL + LANE_DIVIDER_X) / 2; // 190

// --- Flipprar ---
// Halva avståndet (80px) från TABLE_CENTER_X ger samma viloläge-gap som
// innan omcentreringen: ~2.5 bollbredder mellan topparna (128 gav ~2.7,
// 135 gav ~1.9 — för snävt sedan bumper-triangeln blev ensam kvar, 120 gav
// ~3.6 — de siffrorna mättes kring gamla, felaktiga mittpunkten men gapet
// i sig påverkas inte av omcentreringen).
const FLIPPER_HALF_SPAN = 80;
const FLIPPER_PX = TABLE_CENTER_X - FLIPPER_HALF_SPAN; // 110
export const flippers = [
    { px: FLIPPER_PX,                        py: H - 85, angle: 0.15 * Math.PI, restAngle: 0.15 * Math.PI, activeAngle: -0.1 * Math.PI, dir: 'left'  },
    { px: TABLE_CENTER_X + FLIPPER_HALF_SPAN, py: H - 85, angle: 0.85 * Math.PI, restAngle: 0.85 * Math.PI, activeAngle:  1.1 * Math.PI, dir: 'right' },
];

// --- Bumpers ---
// Samma triangel som förut, omcentrerad 20px åt vänster kring TABLE_CENTER_X.
export const bumpers = [
    { x: 120, y: 200, r: 20, flash: 0 },
    { x: 260, y: 180, r: 20, flash: 0 },
    { x: 190, y: 280, r: 20, flash: 0 },
];

// --- Stolpar ---
// Bågens vänstra ände möter väggen med lodrät tangent (cirkelns spets),
// så bollen är redan på väg rakt ner längs väggen exakt där den lämnar
// bågen — utan något i vägen glider den bara rakt ner i vänster outlane.
// Stolpen sitter i den korridoren och kastar in bollen mot bumparna istället.
export const posts = [
    { x: 40, y: 230, r: 14 },
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
// Outlane-guidernas övre ände låg tidigare exakt vid väggen/skiljeväggen
// (avstånd 0) — en boll som höll sig tätt mot den riktiga väggen kunde då
// hamna i en klämma mellan väggen och guidens ändpunkt (bara 9px isär,
// mindre än BALL_R+GUIDE_R=13) och fastna istället för att falla ner i
// outlanen. Några pixlar extra luft löser det.
const OUTLANE_GAP = 30;

export const guides = [
    // Huvudguidernas start är kortad ~30px från väggkanten/skiljeväggen —
    // annars delar de startpunkt exakt med outlane-guiderna nedan, och
    // mellanrummet mellan dem växer då från noll (för smalt för bollen tills
    // det hunnit bre ut sig ordentligt). Vid y=H-150 har det redan blivit
    // minst en bollbredd (18px) på båda sidor.
    { x1: 45,  y1: H - 150, x2: FLIPPER_PX, y2: H - 90 },
    // Spegling av vänsterguiden ovan
    { x1: 335, y1: H - 150, x2: TABLE_CENTER_X + FLIPPER_HALF_SPAN, y2: H - 90 },

    // Utlopp (outlanes) — tajta kanaler tätt intill väggen (vänster) och
    // skjutbanans skiljevägg (höger, för att inte krocka med rampen — den
    // kan därför inte börja lika långt ut som vänstersidans väggkant gör).
    // Hamnar bollen här går den rakt förbi flippern istället för att fångas
    // upp av huvudguiden ovan.
    // y1 = H-115 (inte H-150) så den inte börjar i samma höjd som
    // huvudguiden ovan — vid H-150 ligger de bara ~1px isär horisontellt
    // (samma klämma som vägg/outlane-buggen, fast mellan två guider). Vid
    // H-115 har huvudguiden redan svängt undan ~37px innan outlane-guiden
    // ens existerar, gott och väl mer än de 26px (2×(BALL_R+GUIDE_R)) en
    // boll behöver för att inte nypas mellan dem.
    { x1: WALL + OUTLANE_GAP,           y1: H - 115, x2: WALL + OUTLANE_INSET,           y2: H - 20 },
    { x1: LANE_DIVIDER_X - OUTLANE_GAP, y1: H - 115, x2: LANE_DIVIDER_X - OUTLANE_INSET, y2: H - 20 },

    // Halvcirkelbåge över hela toppen
    ...arcSegments(laneCurveCenter, LANE_CURVE_R, 0, -Math.PI, 32),

    // Skiljevägg för skjutbanan
    { x1: LANE_DIVIDER_X, y1: LANE_TOP, x2: LANE_DIVIDER_X, y2: H },
];

// --- Envägsgrind vid skjutbanans mynning ---
// Tätar öppningen där skjutbanan möter planet ovanför skiljeväggen. Bollen
// passerar den fritt på väg ut (main.js håller den inaktiv tills bollen
// faktiskt lämnat banan), men stängs sedan bakom den — precis som den
// envägstråd riktiga flipperspel har på samma ställe, så en boll redan i
// spel inte kan rulla tillbaka in i röret för en gratis omskjutning.
export const laneGate = { x1: LANE_DIVIDER_X, y1: LANE_TOP, x2: W - WALL - 5, y2: LANE_TOP - 45 };
