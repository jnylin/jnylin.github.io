import { W, H, WALL, BALL_R, FLIP_LEN, FLIP_R, SLINGSHOT_R, PLUNGER_X, PLUNGER_Y, DYING_FRAMES } from './constants.js';
import { flippers, bumpers, guides, slingshots, laneGate, posts, kickbacks } from './entities.js';
import { game } from './state.js';
import { isBallSaveActive, ballSaveFraction } from './ballSave.js';
import { comboMultiplier, comboFraction } from './combo.js';

const canvas = document.getElementById('c');
const ctx    = canvas.getContext('2d');

function drawWalls() {
    ctx.fillStyle = '#1e293b';
    ctx.fillRect(0, 0, WALL, H);
    ctx.fillRect(W - WALL, 0, WALL, H);
    ctx.strokeStyle = '#475569';
    ctx.lineWidth   = 2;
    for (const g of guides) {
        ctx.beginPath();
        ctx.moveTo(g.x1, g.y1);
        ctx.lineTo(g.x2, g.y2);
        ctx.stroke();
    }
    ctx.beginPath();
    ctx.moveTo(laneGate.x1, laneGate.y1);
    ctx.lineTo(laneGate.x2, laneGate.y2);
    ctx.stroke();
}

function drawPosts() {
    for (const p of posts) {
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.r, 0, Math.PI*2);
        ctx.fillStyle   = '#94a3b8';
        ctx.fill();
        ctx.strokeStyle = '#475569';
        ctx.lineWidth   = 2;
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(p.x - p.r*0.3, p.y - p.r*0.3, p.r*0.3, 0, Math.PI*2);
        ctx.fillStyle   = 'rgba(255,255,255,0.5)';
        ctx.fill();
    }
}

function drawBumpers() {
    for (const b of bumpers) {
        const lit = b.flash > 0;
        ctx.beginPath();
        ctx.arc(b.x, b.y, b.r, 0, Math.PI*2);
        ctx.fillStyle   = lit ? '#ff3366'               : '#3d1a2e';
        ctx.fill();
        ctx.strokeStyle = lit ? '#ff6688'               : '#7f1d4a';
        ctx.lineWidth   = 2;
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(b.x - b.r*0.3, b.y - b.r*0.3, b.r*0.3, 0, Math.PI*2);
        ctx.fillStyle   = lit ? 'rgba(255,255,255,0.5)' : 'rgba(255,255,255,0.1)';
        ctx.fill();
    }
}

function drawSlingshots() {
    ctx.lineCap = 'round';
    for (const s of slingshots) {
        const lit = s.flash > 0;
        ctx.strokeStyle = lit ? '#fff35c' : '#8a7a2a';
        ctx.lineWidth   = SLINGSHOT_R * 2;
        ctx.beginPath();
        ctx.moveTo(s.x1, s.y1);
        ctx.lineTo(s.x2, s.y2);
        ctx.stroke();
    }
}

function drawKickbacks() {
    ctx.lineCap = 'round';
    for (const k of kickbacks) {
        const lit = k.flash > 0;
        ctx.strokeStyle = lit ? '#7dd3fc' : '#1e4a5f';
        ctx.lineWidth   = 6;
        ctx.beginPath();
        ctx.moveTo(k.x1, k.y1);
        ctx.lineTo(k.x2, k.y2);
        ctx.stroke();
    }
}

function drawFlipper(f) {
    const tipX = f.px + Math.cos(f.angle)*FLIP_LEN;
    const tipY = f.py + Math.sin(f.angle)*FLIP_LEN;
    ctx.lineCap = 'round';
    ctx.strokeStyle = '#00ffcc';
    ctx.lineWidth   = FLIP_R * 2;
    ctx.beginPath(); ctx.moveTo(f.px, f.py); ctx.lineTo(tipX, tipY); ctx.stroke();
    ctx.globalCompositeOperation = 'destination-over';
    ctx.strokeStyle = '#005544';
    ctx.lineWidth   = FLIP_R * 2 + 2;
    ctx.beginPath(); ctx.moveTo(f.px, f.py); ctx.lineTo(tipX, tipY); ctx.stroke();
    ctx.globalCompositeOperation = 'source-over';
}

function drawBall() {
    const b = game.ball;
    if (!b) return;

    if (b.dying) {
        const t = b.dyingTimer / DYING_FRAMES;
        ctx.globalAlpha = t;
        ctx.beginPath();
        ctx.arc(b.x, b.y, BALL_R * (0.3 + 0.7 * t), 0, Math.PI*2);
        ctx.fillStyle = `hsl(${40 * t}, 100%, 60%)`;
        ctx.fill();
        ctx.globalAlpha = 1;
        return;
    }

    ctx.beginPath();
    ctx.arc(b.x, b.y, BALL_R, 0, Math.PI*2);
    ctx.fillStyle = '#e8c840';
    ctx.fill();
    ctx.beginPath();
    ctx.arc(b.x - 3, b.y - 3, BALL_R * 0.35, 0, Math.PI*2);
    ctx.fillStyle = 'rgba(255,255,255,0.6)';
    ctx.fill();
}

function drawPlunger() {
    if (game.over) return;
    const canCharge = !game.ball || game.ball.waiting;
    if (!canCharge) return;

    // Bollen själv ritas redan av drawBall() när den väntar i röret —
    // rita bara den statiska prickern när det inte finns någon boll alls.
    if (!game.ball) {
        ctx.beginPath();
        ctx.arc(PLUNGER_X, PLUNGER_Y, BALL_R, 0, Math.PI*2);
        ctx.fillStyle = '#e8c840';
        ctx.fill();
        ctx.beginPath();
        ctx.arc(PLUNGER_X - 3, PLUNGER_Y - 3, BALL_R * 0.35, 0, Math.PI*2);
        ctx.fillStyle = 'rgba(255,255,255,0.6)';
        ctx.fill();
    }

    if (game.launchPower > 0) {
        const barH   = 50 * game.launchPower;
        const barX   = PLUNGER_X + BALL_R + 6;
        const barTop = PLUNGER_Y + BALL_R;
        ctx.strokeStyle = '#475569';
        ctx.lineWidth   = 1;
        ctx.strokeRect(barX, barTop - 50, 10, 50);
        ctx.fillStyle = `hsl(${120 - game.launchPower * 120}, 100%, 50%)`;
        ctx.fillRect(barX, barTop - barH, 10, barH);
    }
}

function drawBallSaveIndicator() {
    if (!game.ball || !isBallSaveActive() || game.tilted) return;
    const frac = ballSaveFraction();
    ctx.fillStyle = 'rgba(148,163,184,0.3)';
    ctx.fillRect(WALL, 6, W - 2 * WALL, 4);
    ctx.fillStyle = '#22c55e';
    ctx.fillRect(WALL, 6, (W - 2 * WALL) * frac, 4);
    ctx.fillStyle   = '#22c55e';
    ctx.font        = '11px sans-serif';
    ctx.textAlign   = 'center';
    ctx.fillText('BALL SAVE', W / 2, 24);
}

function drawComboIndicator() {
    const mult = comboMultiplier();
    const frac = comboFraction();
    if (mult <= 1 || frac <= 0) return;
    ctx.textAlign = 'left';
    ctx.fillStyle  = '#38bdf8';
    ctx.font       = 'bold 12px sans-serif';
    ctx.fillText(`COMBO x${mult.toFixed(1)}`, WALL + 4, 20);
    ctx.fillStyle = 'rgba(56,189,248,0.35)';
    ctx.fillRect(WALL + 4, 24, 60 * frac, 3);
}

function drawTiltIndicator() {
    if (!game.tilted) return;
    ctx.fillStyle = '#ef4444';
    ctx.font      = 'bold 16px sans-serif';
    ctx.textAlign = 'center';
    ctx.fillText('TILT', W / 2, 24);
}

export function draw() {
    ctx.clearRect(0, 0, W, H);
    ctx.save();
    ctx.translate(game.shake.x, game.shake.y);
    drawWalls();
    drawPosts();
    drawBumpers();
    drawKickbacks();
    drawSlingshots();
    for (const f of flippers) drawFlipper(f);
    drawBall();
    drawPlunger();
    drawBallSaveIndicator();
    drawComboIndicator();
    ctx.restore();
    drawTiltIndicator();
}
