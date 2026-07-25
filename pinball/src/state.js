import { PLUNGER_X, PLUNGER_Y, LAUNCH_MAX_VY, BALL_SAVE_LAUNCH_POWER } from './constants.js';
import { playLaunch, playDrain, playBallSave } from './audio.js';
import { startBallSave, isBallSaveActive, consumeBallSave } from './ballSave.js';
import { resetCombo } from './combo.js';

const scoreEl = document.getElementById('score');
const ballsEl = document.getElementById('balls');
const hiEl    = document.getElementById('hi');
export const msgEl = document.getElementById('msg');

// --- Rekordpoäng ---
let highScore = parseInt(localStorage.getItem('pinball-hi') || '0', 10);
hiEl.textContent = highScore.toLocaleString();

// --- Speltillstånd ---
// Utökat med shake (för rendering), nudgeCount och tilted
export const game = { 
    score: 0, 
    ballsLeft: 3, 
    over: false, 
    ball: null, 
    launchPower: 0,
    shake: { x: 0, y: 0 },
    tilted: false,
    nudgeCount: 0
};

let nudgeResetTimer = null;

const IDLE_MSG = 'Håll Space för att ladda · släpp för att skjuta';
msgEl.textContent = IDLE_MSG;

export function updateUI() {
    scoreEl.textContent = game.score.toLocaleString();
    ballsEl.textContent = game.ballsLeft;
}

function spawnBall(vy) {
    game.ball = { x: PLUNGER_X, y: PLUNGER_Y, vx: (Math.random() - 0.5) * 2, vy, dying: false, dyingTimer: 0, waiting: false };
}

export function launchBall() {
    game.ballsLeft--;
    updateUI();
    const pwr = game.launchPower;
    game.launchPower = 0;
    spawnBall(-pwr * LAUNCH_MAX_VY);
    startBallSave();
    playLaunch(pwr);
    msgEl.textContent = '← → eller A D för flipprar';
}

// Skjuter iväg samma boll igen efter att den rullat tillbaka ner i röret —
// kostar ingen ny boll eftersom den aldrig kom i spel.
export function relaunchBall() {
    const pwr = game.launchPower;
    game.launchPower = 0;
    const b = game.ball;
    b.x = PLUNGER_X;
    b.y = PLUNGER_Y;
    b.vx = (Math.random() - 0.5) * 2;
    b.vy = -pwr * LAUNCH_MAX_VY;
    b.waiting = false;
    startBallSave();
    playLaunch(pwr);
    msgEl.textContent = '← → eller A D för flipprar';
}

export function ballReturnedToPlunger() {
    msgEl.textContent = 'Bollen rullade tillbaka — håll Space för att ladda om';
}

function autoRelaunch() {
    spawnBall(-BALL_SAVE_LAUNCH_POWER * LAUNCH_MAX_VY);
    msgEl.textContent = 'Bollen räddad! ← → eller A D för flipprar';
}

// --- Nudge & Tilt ---
export function nudgeGame(force) {
    // Om det är TILT, ingen boll finns eller spelet är slut görs ingenting
    if (game.tilted || !game.ball || game.over) return;

    // 1. Påverka kulans hastighet
    game.ball.vx += force.x;
    game.ball.vy += force.y;

    // 2. Sätt offset för visuell skakning på bordet
    game.shake.x = force.x * 2.5;
    game.shake.y = force.y * 2.5;

    // 3. Räkna nudges för Tilt
    game.nudgeCount++;

    if (game.nudgeCount >= 3) {
        game.tilted = true;
        msgEl.textContent = 'TILT! Flipprarna inaktiverade';
    } else {
        msgEl.textContent = `Varning! Nudge (${game.nudgeCount}/3)`;
    }

    // Återställ varningens räknare om spelaren väntar 2 sekunder
    clearTimeout(nudgeResetTimer);
    nudgeResetTimer = setTimeout(() => {
        game.nudgeCount = 0;
        if (!game.tilted && !game.over && game.ball) {
            msgEl.textContent = '← → eller A D för flipprar';
        }
    }, 2000);
}

export function handleDrain() {
    playDrain();

    const wasTilted = game.tilted;

    // Nollställ tilt vid ny kula
    game.tilted = false;
    game.nudgeCount = 0;

    if (!wasTilted && isBallSaveActive()) {
        consumeBallSave();
        playBallSave();
        autoRelaunch();
        return;
    }

    // En tilt förverkar ball-save
    consumeBallSave();
    resetCombo();

    if (game.ballsLeft === 0) {
        game.over = true;
        if (game.score > highScore) {
            highScore = game.score;
            localStorage.setItem('pinball-hi', highScore);
            hiEl.textContent = highScore.toLocaleString();
        }
        msgEl.textContent = `Spelet slut! Poäng: ${game.score.toLocaleString()} — Tryck Space för nytt`;
    } else {
        msgEl.textContent = IDLE_MSG;
    }
}

export function resetGame() {
    game.score = 0; 
    game.ballsLeft = 3; 
    game.over = false; 
    game.ball = null; 
    game.launchPower = 0;
    game.tilted = false;
    game.nudgeCount = 0;
    game.shake = { x: 0, y: 0 };
    consumeBallSave();
    resetCombo();
    updateUI();
    msgEl.textContent = IDLE_MSG;
}
