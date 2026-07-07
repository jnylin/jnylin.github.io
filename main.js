// Injicerar mailto-länken via JS för att undvika enkel scraping av rå HTML.
// Adressen sätts ihop av två delar först vid körning i webbläsaren.
document.addEventListener('DOMContentLoaded', function () {
    var target = document.getElementById('email-link');
    if (!target) return;

    var user = 'jakob.nylin';
    var domain = 'gmail.com';
    var address = user + '@' + domain;

    var link = document.createElement('a');
    link.href = 'mailto:' + address;
    link.textContent = address;

    target.appendChild(link);
});

// Möjlighet till utskrift
document
    .getElementById("print-button")
    ?.addEventListener("click", () => window.print());
