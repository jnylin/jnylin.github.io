// The per-frame simulation (flipper angles, gravity, collisions, drain,
// scoring, tilt/nudge, ball lifecycle) now runs in rust-core, driven
// through stepFrame() — see world.rs for the ported logic and state.js for
// how its JSON events turn into sounds/particles/messages. This file is
// just the browser-side loop: figure out dt, gather input, hand it to the
// engine, draw whatever it left in `game`/the entity arrays.
import { keys, leftDown, rightDown } from './input.js';
import { tickParticles } from './particles.js';
import { stepFrame } from './state.js';
import { draw } from './render.js';

function update(dtFactor) {
    tickParticles(); // purely visual, not part of the ported simulation
    const charging = !!keys['Space'];
    stepFrame(dtFactor, leftDown(), rightDown(), charging);
}

let lastTime = performance.now();

function loop(currentTime) {
    // Normalisera beräkningen mot 60 FPS (16.67ms per frame)
    const dt = (currentTime - lastTime) / 1000;
    lastTime = currentTime;

    // Kapa extremvärden (t.ex. om man byter flik i webbläsaren)
    const clampedDt = Math.min(dt, 0.1);
    const dtFactor = clampedDt * 60;

    update(dtFactor);
    draw();

    requestAnimationFrame(loop);
}

// Starta spelloopen
requestAnimationFrame(loop);
