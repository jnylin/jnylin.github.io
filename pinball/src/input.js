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

export const leftDown  = () => keys['ArrowLeft']  || keys['KeyA'];
export const rightDown = () => keys['ArrowRight'] || keys['KeyD'];
