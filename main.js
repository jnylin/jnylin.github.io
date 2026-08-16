document.addEventListener('DOMContentLoaded', function () {
    // 1. Injicerar mailto-länken via JS för att undvika enkel scraping av rå HTML.
    const target = document.getElementById('email-link');
    if (target) {
        const user = 'jakob.nylin';
        const domain = 'gmail.com';
        const address = user + '@' + domain;

        const link = document.createElement('a');
        link.href = 'mailto:' + address;
        link.textContent = address;

        target.appendChild(link);
    }

    // 2. Möjlighet till utskrift
    document
        .getElementById("print-button")
        ?.addEventListener("click", () => window.print());

    // 3. Dynamisk generering av JSON-LD för DRY språkhantering
    injectJsonLd();
});

function injectJsonLd() {
    const jsonLdData = {
        sv: {
            jobTitle: "Systembibliotekarie och utvecklare",
            degrees: [
                {
                    name: "Magisterexamen i biblioteks- och informationsvetenskap",
                    description: "Inriktning kunskapsorganisation, med särskilt intresse för information retrieval"
                },
                {
                    name: "Kandidatexamen i allmän språkvetenskap",
                    description: "Fokus på semantik och syntax"
                }
            ]
        },
        en: {
            jobTitle: "Systems Librarian & Developer",
            degrees: [
                {
                    name: "Master's Degree in Library and Information Science",
                    description: "Specialization in knowledge organization, with a particular interest in information retrieval"
                },
                {
                    name: "Bachelor's Degree in General Linguistics",
                    description: "Focus on semantics and syntax"
                }
            ]
        }
    };

    // Läs av språk från <html lang="...">
    const lang = document.documentElement.lang || 'sv';
    const localized = jsonLdData[lang] || jsonLdData.sv;

    const schema = {
        "@context": "https://schema.org",
        "@type": "Person",
        "name": "Jakob Nylin Nilsson",
        "jobTitle": localized.jobTitle,
        "url": window.location.href,
        "sameAs": [
            "https://gitlab.com/lnu-ub"
        ],
        "worksFor": {
            "@type": "Organization",
            "name": "Linnéuniversitetet"
        },
        "hasCredential": localized.degrees.map(deg => ({
            "@type": "EducationalOccupationalCredential",
            "credentialCategory": "degree",
            "name": deg.name,
            "description": deg.description
        })),
        "knowsAbout": [
            "JavaScript", "Node.js", "Ruby", "Bash", "SQL",
            "OIDC", "OAuth2", "SAML", "Keycloak", "Shibboleth", "REST",
            "Alma", "FOLIO", "Primo", "VuFind", "Linux", "Apache",
            "Git", "GitLab CI/CD", "Playwright", "axe-core"
        ]
    };

    const script = document.createElement('script');
    script.type = 'application/ld+json';
    script.text = JSON.stringify(schema);
    document.head.appendChild(script);
}
