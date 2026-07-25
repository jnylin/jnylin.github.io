// --- Combo-multiplikator ---
// Självständig timer, precis som ballSave.js: varje registrerad träff inom
// fönstret höjer multiplikatorn ett steg. Går fönstret ut innan nästa träff
// börjar kedjan om från 1x. Känner inte till game/ball-objekten.
import { COMBO_WINDOW_FRAMES, COMBO_STEP, COMBO_MAX } from './constants.js';

let multiplier = 1;
let framesLeft = 0;

export function registerHit() {
    multiplier = framesLeft > 0 ? Math.min(COMBO_MAX, multiplier + COMBO_STEP) : 1;
    framesLeft = COMBO_WINDOW_FRAMES;
    return multiplier;
}

export function tickCombo()      { if (framesLeft > 0) framesLeft--; }
export function resetCombo()     { multiplier = 1; framesLeft = 0; }
export function comboMultiplier() { return multiplier; }
export function comboFraction()   { return framesLeft / COMBO_WINDOW_FRAMES; }
