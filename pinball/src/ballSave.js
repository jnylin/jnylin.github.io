// --- Ball-save ---
// Självständig timer: startas vid utskjutning, och om bollen drainar innan
// den löper ut räddas den (se handleDrain i state.js). Håller inget annat
// tillstånd och känner inte till game/ball-objekten.
import { BALL_SAVE_FRAMES } from './constants.js';

let framesLeft = 0;

export function startBallSave()     { framesLeft = BALL_SAVE_FRAMES; }
export function consumeBallSave()   { framesLeft = 0; }
export function tickBallSave()      { if (framesLeft > 0) framesLeft--; }
export function isBallSaveActive()  { return framesLeft > 0; }
export function ballSaveFraction()  { return framesLeft / BALL_SAVE_FRAMES; }
