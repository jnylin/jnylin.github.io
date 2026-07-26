import { initAudio, playFlipperHit } from './audio.js';
import { game, resetGame, launchBall, relaunchBall, nudgeGame } from './state.js';

// --- Inmatning ---
export const keys = {};

document.addEventListener('keydown', e => {
    // Förhindra standard-scrolling med pilgörntor och blanksteg
    if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown', 'Space'].includes(e.code)) {
        e.preventDefault();
    }
  
    if (!keys[e.code]) {
        initAudio();

        // Flipprar
        if (e.code === 'ArrowLeft'  || e.code === 'KeyA') playFlipperHit();
        if (e.code === 'ArrowRight' || e.code === 'KeyD') playFlipperHit();

        // Starta om
        if (e.code === 'Space' && game.over) resetGame();

        // --- Nudge / Tilt ---
        // Stöt åt vänster (skuffar spelet åt vänster -> kulan rör sig åt höger)
        if (e.code === 'KeyZ') {
            nudgeGame({ x: 2.5, y: -0.5 });
        }
        // Stöt åt höger (skuffar spelet åt höger -> kulan rör sig åt vänster)
        if (e.code === 'KeyX' || e.code === 'Slash') {
            nudgeGame({ x: -2.5, y: -0.5 });
        }
        // Stöt uppåt (skuffar spelet framåt -> kulan får lite extra fart uppåt)
        if (e.code === 'KeyN' || e.code === 'ArrowUp') {
            nudgeGame({ x: 0, y: -3.0 });
        }
    }
    
    keys[e.code] = true;
});

document.addEventListener('keyup', e => {
    keys[e.code] = false;

    // Skjuta iväg kulan
    if (e.code === 'Space' && !game.over && game.launchPower >= 0.1) {
        if (!game.ball && game.ballsLeft > 0) {
            launchBall();
        } else if (game.ball && game.ball.waiting) {
            relaunchBall();
        }
    }
});

// --- Touch-styrning (mobil) ---
// Skärmen delas i två halvor: vänster halva styr vänster flipper, höger
// halva styr höger flipper. Ett nedåtdrag laddar plungern proportionellt
// mot dragets längd — släpp fingret för att skjuta, precis som ett
// riktigt plungerhandtag.
const DRAG_THRESHOLD = 12; // px innan ett tryck räknas som ett drag nedåt
const DRAG_RANGE     = 90; // px drag för full kraft

const touches = new Map(); // identifier -> { side, startX, startY, mode }

function touchSide(clientX) {
    return clientX < window.innerWidth / 2 ? 'left' : 'right';
}

function touchSideActive(side) {
    for (const state of touches.values()) {
        if (state.mode === 'flipper' && state.side === side) return true;
    }
    return false;
}

function releaseTouch(id) {
    const state = touches.get(id);
    if (!state) return;
    touches.delete(id);

    if (state.mode === 'drag' && !game.over && game.launchPower >= 0.1) {
        if (!game.ball && game.ballsLeft > 0) {
            launchBall();
        } else if (game.ball && game.ball.waiting) {
            relaunchBall();
        }
    }
}

document.addEventListener('touchstart', e => {
    e.preventDefault();
    initAudio();

    if (game.over) resetGame();

    for (const t of e.changedTouches) {
        touches.set(t.identifier, { side: touchSide(t.clientX), startX: t.clientX, startY: t.clientY, mode: 'flipper' });
        playFlipperHit();
    }
}, { passive: false });

document.addEventListener('touchmove', e => {
    e.preventDefault();

    for (const t of e.changedTouches) {
        const state = touches.get(t.identifier);
        if (!state) continue;

        const dy = t.clientY - state.startY;
        const dx = t.clientX - state.startX;

        if (state.mode === 'flipper' && dy > DRAG_THRESHOLD && dy > Math.abs(dx)) {
            state.mode = 'drag';
        }

        if (state.mode === 'drag') {
            const chargeable = (!game.ball || game.ball.waiting) && !game.over;
            if (chargeable) {
                game.launchPower = Math.max(0, Math.min(1, (dy - DRAG_THRESHOLD) / DRAG_RANGE));
            }
        }
    }
}, { passive: false });

document.addEventListener('touchend', e => {
    e.preventDefault();
    for (const t of e.changedTouches) releaseTouch(t.identifier);
}, { passive: false });

document.addEventListener('touchcancel', e => {
    for (const t of e.changedTouches) releaseTouch(t.identifier);
}, { passive: false });

export const leftDown  = () => keys['ArrowLeft']  || keys['KeyA'] || touchSideActive('left');
export const rightDown = () => keys['ArrowRight'] || keys['KeyD'] || touchSideActive('right');
