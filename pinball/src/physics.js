import { BALL_R, GUIDE_R, FLIP_R, FLIP_LEN, SLINGSHOT_R, SLINGSHOT_KICK, SLINGSHOT_FLASH_FRAMES } from './constants.js';
import { leftDown, rightDown } from './input.js';
import { playBumperHit, playSlingshotHit } from './audio.js';
import { game, updateUI } from './state.js';
import { registerHit } from './combo.js';

// --- Hjälpfunktioner för fysik ---
export function closestPointOnSegment(px, py, ax, ay, bx, by) {
    const dx = bx - ax, dy = by - ay;
    const t  = Math.max(0, Math.min(1, ((px - ax)*dx + (py - ay)*dy) / (dx*dx + dy*dy)));
    return { x: ax + t*dx, y: ay + t*dy };
}

export function reflectBallOffSegment(ball, ax, ay, bx, by, segR, onHit) {
    const cp   = closestPointOnSegment(ball.x, ball.y, ax, ay, bx, by);
    const ex   = ball.x - cp.x, ey = ball.y - cp.y;
    const dist = Math.hypot(ex, ey);
    const min  = BALL_R + segR;
    if (dist >= min || dist < 0.01) return;
    const nx = ex/dist, ny = ey/dist;
    ball.x = cp.x + nx * min;
    ball.y = cp.y + ny * min;
    onHit(ball, nx, ny, ball.vx*nx + ball.vy*ny);
}

// --- Kollisioner ---
export function collideFlipper(ball, f) {
    const tipX = f.px + Math.cos(f.angle) * FLIP_LEN;
    const tipY = f.py + Math.sin(f.angle) * FLIP_LEN;

    reflectBallOffSegment(ball, f.px, f.py, tipX, tipY, FLIP_R, (b, nx, ny, dot) => {
        // En "aktiv flick" kräver att flippern faktiskt är mitt i sin rörelse uppåt
        const isFlick = f.isMovingUp;

        if (isFlick) {
            // Aktiv flick: hela vektorn boostas rejält
            b.vx = (b.vx - 2 * dot * nx) * 1.8;
            b.vy = (b.vy - 2 * dot * ny) * 1.8;

            if (b.vy > -2) b.vy -= 4;
        } else {
            // Stillastående flipper (oavsett om den hålls uppe i toppläget eller ligger i viloläge):
            // Bara normalkomponenten dämpas så att bollen kan rulla snyggt längs flippern.
            b.vx -= 1.4 * dot * nx;
            b.vy -= 1.4 * dot * ny;
        }
    });
}

export function collideBumper(ball, b) {
    const dx   = ball.x - b.x, dy = ball.y - b.y;
    const dist = Math.hypot(dx, dy);
    const min  = BALL_R + b.r;
    if (dist >= min || dist < 0.01) return;
    const mult = registerHit();
    game.score += Math.round(50 * mult);
    updateUI();
    b.flash = 18;
    playBumperHit();
    const nx    = dx/dist, ny = dy/dist;
    ball.x      = b.x + nx * min;
    ball.y      = b.y + ny * min;
    // 15% dämpning per studs — annars konserverar bumpern farten nästan
    // exakt och bollen kan studsa runt bumper-triangeln snabbt utan att
    // tappa fart (0.9 kändes för kraftigt över flera studsar i klustret).
    const speed = Math.max(Math.hypot(ball.vx, ball.vy) * 0.85, 6);
    ball.vx     = nx * speed;
    ball.vy     = ny * speed;
}

// En stum stolpe (ingen poäng/ljud) som studsar bollen radiellt bort från
// sin mittpunkt, till skillnad från en guide som bara håller bollen på ena
// sidan av en linje. Det gör den till en riktig "spets" som kastar bollen
// in mot planet oavsett vilken vinkel den kommer in i, snarare än att bara
// bilda ännu en korridor bollen kan glida rakt ner genom.
export function collidePost(ball, p) {
    const dx   = ball.x - p.x, dy = ball.y - p.y;
    const dist = Math.hypot(dx, dy);
    const min  = BALL_R + p.r;
    if (dist >= min || dist < 0.01) return;
    const nx    = dx/dist, ny = dy/dist;
    ball.x      = p.x + nx * min;
    ball.y      = p.y + ny * min;
    const speed = Math.hypot(ball.vx, ball.vy) * 0.9;
    ball.vx     = nx * speed;
    ball.vy     = ny * speed;
}

export function collideGuide(ball, g) {
    reflectBallOffSegment(ball, g.x1, g.y1, g.x2, g.y2, GUIDE_R, (b, nx, ny, dot) => {
        if (dot < 0) {
            // Bara normalkomponenten studsar/dämpas (0.7 = restitution) —
            // tangentialfarten (rullningen längs guiden) lämnas orörd, se
            // motsvarande kommentar i collideFlipper.
            b.vx -= 1.7 * dot * nx;
            b.vy -= 1.7 * dot * ny;
        }
    });
}

export function collideSlingshot(ball, s) {
    reflectBallOffSegment(ball, s.x1, s.y1, s.x2, s.y2, SLINGSHOT_R, (b, nx, ny, dot) => {
        if (dot >= 0) return;
        b.vx -= 2 * dot * nx;
        b.vy -= 2 * dot * ny;
        const speed = Math.hypot(b.vx, b.vy) + SLINGSHOT_KICK;
        const ang   = Math.atan2(b.vy, b.vx);
        b.vx = Math.cos(ang) * speed;
        b.vy = Math.sin(ang) * speed;
        s.flash = SLINGSHOT_FLASH_FRAMES;
        const mult = registerHit();
        game.score += Math.round(10 * mult);
        updateUI();
        playSlingshotHit();
    });
}
