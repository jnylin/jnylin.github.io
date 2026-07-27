import { G, MAX_SPEED, BALL_R, WALL, W, H, SUBSTEP_DIST, FLIP_SPEED, LAUNCH_RATE, DYING_FRAMES, LANE_DIVIDER_X, LAUNCH_MAX_VY, PLUNGER_X, PLUNGER_Y } from './constants.js';
import { flippers, bumpers, guides, slingshots, laneGate, posts } from './entities.js';
import { collideFlipper, collideBumper, collideGuide, collideSlingshot, collidePost } from './physics.js';
import { game, handleDrain, ballReturnedToPlunger, launchBall } from './state.js';
import { keys, leftDown, rightDown } from './input.js';
import { tickBallSave } from './ballSave.js';
import { tickCombo } from './combo.js';
import { draw } from './render.js';

// --- Uppdatera speltillstånd ---
function updateFlippers(dtFactor = 1) {
    for (const f of flippers) {
        const pressing = !game.tilted && ((f.dir === 'left' && leftDown()) || (f.dir === 'right' && rightDown()));
        const prevAngle = f.angle;

        if (pressing) {
            f.angle += FLIP_SPEED * dtFactor * (f.dir === 'left' ? -1 : 1);
            f.angle  = f.dir === 'left'
                ? Math.max(f.activeAngle, f.angle)
                : Math.min(f.activeAngle, f.angle);
        } else {
            f.angle += FLIP_SPEED * 0.6 * dtFactor * (f.dir === 'left' ? 1 : -1);
            f.angle  = f.dir === 'left'
                ? Math.min(f.restAngle, f.angle)
                : Math.max(f.restAngle, f.angle);
        }

        // Flippern klassas som "i aktiv rörelse uppåt" BARA om vinkeln faktiskt ändrades medan man tryckte
        f.isMovingUp = pressing && (f.angle !== prevAngle);
    }
}

function decayShake() {
    game.shake.x *= 0.8;
    game.shake.y *= 0.8;
    if (Math.abs(game.shake.x) < 0.05) game.shake.x = 0;
    if (Math.abs(game.shake.y) < 0.05) game.shake.y = 0;
}

// Fångar en boll som inte orkade hela vägen upp genom skjutbanan och föll
// tillbaka, istället för att låta den falla vidare ut genom botten (drain).
function checkLaunchReturn(b) {
    const inLane = b.x > LANE_DIVIDER_X;
    if (inLane && b.y >= PLUNGER_Y && b.vy >= 0) {
        b.x = PLUNGER_X;
        b.y = PLUNGER_Y;
        b.vx = 0;
        b.vy = 0;
        b.waiting = true;
        ballReturnedToPlunger();
        return true;
    }
    return false;
}

function update(dtFactor = 1) {
    updateFlippers();
    decayShake();

    // 1. Ladda skottet när Space hålls nere (Tangenthantering)
    const chargeable = (!game.ball || game.ball.waiting) && !game.over;
    if (chargeable && keys['Space']) {
        game.launchPower = Math.min(1, game.launchPower + LAUNCH_RATE * dtFactor);
    }

    const b = game.ball;
    if (!b || b.waiting) return;

    tickBallSave();
    tickCombo();

    if (b.dying) {
        b.dyingTimer--;
        if (b.dyingTimer <= 0) { game.ball = null; handleDrain(); }
        return;
    }

    // Applicera gravitations- och hastighetsfaktor för dt (så farten blir stabil i 60/120Hz)
    b.vy += G * dtFactor;
    b.vx = Math.max(-MAX_SPEED, Math.min(MAX_SPEED, b.vx));
    
    // I skjutbanan tillåts högre fart så att en fullkraftsskjutning inte
    // klipps ner till det generella hastighetstaket innan den nått toppen.
    const vyCap = b.x > LANE_DIVIDER_X ? LAUNCH_MAX_VY : MAX_SPEED;
    b.vy = Math.max(-vyCap, Math.min(vyCap, b.vy));

    const steps = Math.max(1, Math.ceil(Math.max(Math.abs(b.vx), Math.abs(b.vy)) / SUBSTEP_DIST));
    for (let s = 0; s < steps; s++) {
        b.x += (b.vx * dtFactor) / steps;
        b.y += (b.vy * dtFactor) / steps;

        // Sidoväggar
        if (b.x - BALL_R < WALL)     { b.vx =  Math.abs(b.vx) * 0.75; b.x = WALL + BALL_R; }
        if (b.x + BALL_R > W - WALL) { b.vx = -Math.abs(b.vx) * 0.75; b.x = W - WALL - BALL_R; }

        // Grinden stängs först nästa substep efter att bollen lämnat banan,
        // så den inte kan råka stänga på sig själv i samma steg den passerar.
        const gateActive = b.hasEscaped;
        if (!b.hasEscaped && b.x < LANE_DIVIDER_X) b.hasEscaped = true;
        if (gateActive) collideGuide(b, laneGate);

        for (const bmp of bumpers)    collideBumper(b, bmp);
        for (const p   of posts)      collidePost(b, p);
        for (const sl  of slingshots) collideSlingshot(b, sl);
        for (const g   of guides)     collideGuide(b, g);
        for (const f   of flippers)   collideFlipper(b, f);
    }
    
    for (const bmp of bumpers)    { if (bmp.flash > 0) bmp.flash--; }
    for (const sl  of slingshots) { if (sl.flash  > 0) sl.flash--;  }

    if (checkLaunchReturn(b)) return;

    if (b.y > H + 20) {
        b.dying      = true;
        b.dyingTimer = DYING_FRAMES;
    }
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
