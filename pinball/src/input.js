import { initAudio, playFlipperHit } from './audio.js';
import { game, resetGame, triggerLaunch, nudgeGame, setLaunchPower } from './state.js';

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
    if (e.code === 'Space') {
        triggerLaunch();
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

    if (state.mode === 'drag') {
        triggerLaunch();
    }
}

// --- Tilt via mobilens gyro/accelerometer ---
// Ett kraftigt ryck i telefonen ger samma nudge/tilt-hantering som Z/X/N
// på tangentbordet — återanvänder nudgeGame() istället för en egen
// tilt-räknare, så varningar/3-strikes/skärmskakning fungerar identiskt.
let lastAcc = null;
let gyroInitialized = false;
const SHAKE_THRESHOLD = 12; // m/s² — okalibrerad, justera efter test på riktig mobil

function handleMotion(event) {
    const acc = event.accelerationIncludingGravity;
    if (!acc || acc.x === null || acc.y === null) return;

    if (lastAcc) {
        const deltaX = acc.x - lastAcc.x;
        const deltaY = acc.y - lastAcc.y;
        if (Math.abs(deltaX) + Math.abs(deltaY) > SHAKE_THRESHOLD) {
            nudgeGame({ x: deltaX * 0.5, y: deltaY * 0.5 });
        }
    }
    lastAcc = { x: acc.x, y: acc.y };
}

// iOS kräver explicit tillstånd från ett användar-tap, Android/övriga
// webbläsare lyssnar direkt — anropas första gången spelaren rör canvasen.
function initGyroTilt() {
    if (gyroInitialized) return;
    gyroInitialized = true;

    if (typeof DeviceMotionEvent !== 'undefined' && typeof DeviceMotionEvent.requestPermission === 'function') {
        DeviceMotionEvent.requestPermission()
            .then(response => {
                if (response === 'granted') window.addEventListener('devicemotion', handleMotion);
            })
            .catch(() => {});
    } else if (typeof DeviceMotionEvent !== 'undefined') {
        window.addEventListener('devicemotion', handleMotion);
    }
}

document.addEventListener('touchstart', e => {
    e.preventDefault();
    initAudio();
    initGyroTilt();

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
                setLaunchPower(Math.max(0, Math.min(1, (dy - DRAG_THRESHOLD) / DRAG_RANGE)));
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
