// --- Konstanter ---
export const W               = 420, H         = 660;
export const G                = 0.3;
export const MAX_SPEED       = 20;
export const BALL_R          = 9;
export const WALL            = 16;
export const FLIP_LEN        = 65;
export const FLIP_R          = 7;
export const FLIP_SPEED      = 0.18;
export const GUIDE_R         = 4;
export const SUBSTEP_DIST    = 6;
export const LANE_W          = 40;
export const LANE_DIVIDER_X  = W - WALL - LANE_W;

// Taket som en halvcirkel över hela spelfältets bredd
export const PLAYFIELD_W     = W - 2 * WALL;
export const LANE_CURVE_R    = PLAYFIELD_W / 2;
export const LANE_TOP        = LANE_CURVE_R;
export const laneCurveCenter = { x: W / 2, y: LANE_TOP };

export const PLUNGER_X       = (LANE_DIVIDER_X + (W - WALL)) / 2;
export const PLUNGER_Y       = H - 60;
// Egen hastighetsgräns för utskjutning, frikopplad från MAX_SPEED — annars
// klipps kraften ner till det generella taket innan bollen hunnit ta sig
// upp genom skjutbanan (som kräver ~15.6 i starthastighet för att nå toppen).
export const LAUNCH_MAX_VY   = 32;
export const LAUNCH_RATE     = 1 / 70;
export const DYING_FRAMES    = 25;

// --- Slingshots ---
export const SLINGSHOT_R            = 6;
export const SLINGSHOT_KICK         = 6;
export const SLINGSHOT_FLASH_FRAMES = 12;

// --- Ball-save ---
export const BALL_SAVE_FRAMES       = 180; // ~3s @60fps
export const BALL_SAVE_LAUNCH_POWER = 0.75;

// --- Combo-multiplikator ---
export const COMBO_WINDOW_FRAMES = 90; // ~1.5s @60fps mellan träffar
export const COMBO_STEP          = 0.5;
export const COMBO_MAX           = 4;
