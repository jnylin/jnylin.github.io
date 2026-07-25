// --- Ljud ---
let audioCtx = null;
export function initAudio() { if (!audioCtx) audioCtx = new AudioContext(); }

function tone(freq, endFreq, dur, type, vol) {
    if (!audioCtx) return;
    const osc  = audioCtx.createOscillator();
    const gain = audioCtx.createGain();
    osc.connect(gain); gain.connect(audioCtx.destination);
    osc.type = type;
    osc.frequency.setValueAtTime(freq, audioCtx.currentTime);
    if (endFreq !== freq) osc.frequency.exponentialRampToValueAtTime(endFreq, audioCtx.currentTime + dur);
    gain.gain.setValueAtTime(vol, audioCtx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, audioCtx.currentTime + dur);
    osc.start(); osc.stop(audioCtx.currentTime + dur);
}

export const playBumperHit    = ()    => tone(700, 300, 0.12, 'square',   0.15);
export const playFlipperHit   = ()    => tone(90,  55,  0.07, 'sawtooth', 0.12);
export const playDrain        = ()    => tone(380, 80,  0.7,  'sine',     0.25);
export const playLaunch       = (pwr) => tone(80 + pwr * 180, 40, 0.25, 'sawtooth', 0.15 * pwr);
export const playSlingshotHit = ()    => tone(520, 200, 0.1,  'triangle', 0.18);
export const playBallSave     = ()    => tone(300, 900, 0.35, 'sine',     0.2);
