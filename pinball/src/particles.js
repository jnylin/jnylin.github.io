// --- Partiklar & score-popups ---
// Ren visuell juice: känner inte till game/ball-objekten, bara sin egen lista
// (samma mönster som combo.js och ballSave.js).
let sparks = [];
let popups = [];

export function spawnSpark(x, y, color, count = 10) {
    for (let i = 0; i < count; i++) {
        const angle = Math.random() * Math.PI * 2;
        const speed = 1.5 + Math.random() * 3.5;
        sparks.push({
            x, y,
            vx: Math.cos(angle) * speed,
            vy: Math.sin(angle) * speed,
            life: 1,
            color,
        });
    }
}

export function spawnScorePopup(x, y, amount) {
    popups.push({ x, y, amount, life: 1 });
}

export function tickParticles() {
    for (const s of sparks) {
        s.x  += s.vx;
        s.y  += s.vy;
        s.vx *= 0.92;
        s.vy *= 0.92;
        s.life -= 0.05;
    }
    sparks = sparks.filter(s => s.life > 0);

    for (const p of popups) {
        p.y    -= 0.6;
        p.life -= 0.02;
    }
    popups = popups.filter(p => p.life > 0);
}

export function drawParticles(ctx) {
    for (const s of sparks) {
        ctx.globalAlpha = Math.max(0, s.life);
        ctx.beginPath();
        ctx.arc(s.x, s.y, 2, 0, Math.PI * 2);
        ctx.fillStyle = s.color;
        ctx.fill();
    }
    ctx.globalAlpha = 1;

    ctx.textAlign = 'center';
    ctx.font      = 'bold 12px sans-serif';
    for (const p of popups) {
        ctx.globalAlpha = Math.max(0, p.life);
        ctx.fillStyle    = '#fff';
        ctx.fillText(`+${p.amount}`, p.x, p.y);
    }
    ctx.globalAlpha = 1;
}
