// The game engine now lives in rust-core (see wasm_api.rs, world.rs,
// state.rs) — this module owns the one PinballApi instance, mirrors its
// state into the plain `game` object render.js/audio.js already read (so
// those files needed no changes), and translates the JSON events it
// returns into the same sounds/particles/message text the old pure-JS
// version produced directly.
import { flippers, bumpers, slingshots, kickbacks } from './entities.js';
import { playBumperHit, playSlingshotHit, playDrain, playLaunch, playBallSave } from './audio.js';
import { spawnSpark, spawnScorePopup } from './particles.js';
import { loadWasm } from './wasmBridge.js';

const scoreEl = document.getElementById('score');
const ballsEl = document.getElementById('balls');
const hiEl    = document.getElementById('hi');
export const msgEl = document.getElementById('msg');

// --- Rekordpoäng ---
// rust-core never touches localStorage (see state.rs's doc comment) — this
// stays entirely JS's job, same as before.
let highScore = parseInt(localStorage.getItem('pinball-hi') || '0', 10);
hiEl.textContent = highScore.toLocaleString();

const PinballApi = await loadWasm();
const api = new PinballApi(highScore);

// --- Speltillstånd ---
// En JS-spegel av wasm-corets tillstånd, synkad varje bildruta (se
// syncGame/syncEntities) — render.js och audio.js läser fortfarande bara
// det här objektet, precis som innan porteringen.
export const game = {
    score: 0,
    ballsLeft: 3,
    over: false,
    ball: null,
    launchPower: 0,
    shake: { x: 0, y: 0 },
    tilted: false,
};

const IDLE_MSG = 'Håll Space eller dra nedåt för att ladda · släpp för att skjuta';
msgEl.textContent = IDLE_MSG;

export function updateUI() {
    scoreEl.textContent = game.score.toLocaleString();
    ballsEl.textContent = game.ballsLeft;
}

export function isBallSaveActive() { return api.is_ball_save_active(); }
export function ballSaveFraction() { return api.ball_save_fraction(); }
export function comboMultiplier()  { return api.combo_multiplier(); }
export function comboFraction()    { return api.combo_fraction(); }

// For input.js's touch-drag charge gesture, which sets launch power
// directly from drag distance rather than accumulating it over time the
// way holding Space does inside stepFrame() — a direct write that a
// naive "sync FROM wasm every frame" design would otherwise clobber on
// the very next frame.
export function setLaunchPower(value) {
    api.set_launch_power(value);
    game.launchPower = api.launch_power();
}

function syncGame() {
    game.score       = api.score();
    game.ballsLeft   = api.balls_left();
    game.over        = api.over();
    game.launchPower = api.launch_power();
    game.tilted      = api.tilted();
    game.shake.x     = api.shake_x();
    game.shake.y     = api.shake_y();

    game.ball = api.ball_present()
        ? {
            x: api.ball_x(),
            y: api.ball_y(),
            dying: api.ball_dying(),
            dyingTimer: api.ball_dying_timer(),
            waiting: api.ball_waiting(),
        }
        : null;

    updateUI();
}

function syncEntities() {
    for (let i = 0; i < flippers.length; i++) {
        flippers[i].angle      = api.flipper_angle(i);
        flippers[i].isMovingUp = api.flipper_is_moving_up(i);
    }
    for (let i = 0; i < bumpers.length; i++)    bumpers[i].flash    = api.bumper_flash(i);
    for (let i = 0; i < slingshots.length; i++) slingshots[i].flash = api.slingshot_flash(i);
    for (let i = 0; i < kickbacks.length; i++)  kickbacks[i].flash  = api.kickback_flash(i);
}

function applyDrainEvent(ev) {
    playDrain();
    if (ev.type === 'BallSaved') {
        playBallSave();
        msgEl.textContent = 'Bollen räddad! ← → eller A D för flipprar';
    } else if (ev.type === 'GameOver') {
        if (ev.is_new_high_score) {
            highScore = ev.score;
            localStorage.setItem('pinball-hi', highScore);
            hiEl.textContent = highScore.toLocaleString();
        }
        msgEl.textContent = `Spelet slut! Poäng: ${ev.score.toLocaleString()} — Tryck Space för nytt`;
    } else {
        msgEl.textContent = IDLE_MSG;
    }
}

function applyStepEvents(eventsJson) {
    for (const ev of JSON.parse(eventsJson)) {
        switch (ev.event) {
            case 'BumperHit':
                playBumperHit();
                if (ev.special) spawnSpark(ev.x, ev.y, '#c084fc');
                spawnScorePopup(ev.x, ev.y - 18, ev.gain);
                break;
            case 'SlingshotHit':
                playSlingshotHit();
                spawnScorePopup(ev.x, ev.y - 18, ev.gain);
                break;
            case 'ReturnedToPlunger':
                msgEl.textContent = 'Bollen rullade tillbaka — håll Space för att ladda om';
                break;
            case 'NudgeReset':
                msgEl.textContent = '← → eller A D för flipprar';
                break;
            case 'Drained':
                applyDrainEvent(ev);
                break;
        }
    }
}

// Called once per animation frame from main.js's loop.
export function stepFrame(dtFactor, leftPressing, rightPressing, charging) {
    const eventsJson = api.step(dtFactor, leftPressing, rightPressing, charging, Math.random());
    applyStepEvents(eventsJson);
    syncGame();
    syncEntities();
}

export function triggerLaunch() {
    const outcome = JSON.parse(api.trigger_launch(Math.random()));
    if (outcome) {
        playLaunch(outcome.power);
        msgEl.textContent = '← → eller A D för flipprar';
        syncGame();
    }
}

export function nudgeGame(force) {
    const outcome = JSON.parse(api.nudge(force.x, force.y));
    if (outcome?.type === 'Tilted') {
        msgEl.textContent = 'TILT! Flipprarna inaktiverade';
    } else if (outcome?.type === 'Warning') {
        msgEl.textContent = `Varning! Nudge (${outcome.count}/3)`;
    }
    if (outcome) syncGame();
}

export function resetGame() {
    api.reset();
    syncGame();
    syncEntities();
    msgEl.textContent = IDLE_MSG;
}
