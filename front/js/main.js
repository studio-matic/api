const host = window.location.hostname;
let baseUrl, hostingPrefix;
if (
    host === "localhost" ||
    /^127\./.test(host) ||
    host === "0.0.0.0" ||
    host === "[::1]" ||
    host === "[::]" ||
    /^10\./.test(host) ||
    /^192\.168\./.test(host) ||
    /^172\.(1[6-9]|2\d|3[0-1])\./.test(host) ||
    /^\[?(fc|fd)[0-9a-fA-F:]+\]?$/.test(host)
) {
    baseUrl = `http://${host}:3000`;
    hostingPrefix = ""
} else {
    baseUrl = "https://api.studio-matic.org";
    hostingPrefix = ""
}

async function checkBackend() {
    const el = document.getElementById("status");
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 3000);
    try {
        const r = await fetch(`${baseUrl}/health`, { method: "GET", signal: controller.signal });
        clearTimeout(timeoutId);
        el.innerText = r.ok ? "backend online ✅" : "backend offline ❌";
    } catch (_) {
        clearTimeout(timeoutId);
        el.innerText = "backend offline ❌"
    }
}
checkBackend()
setInterval(checkBackend, 10000)

async function welc() {
    const el = document.getElementById("welc");
    const res = await fetch(`${baseUrl}/users/me`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    if (res.ok) {
        const data = await res.json();
        el.innerText = el.innerText + " " + data.email;
    }
}

async function redirLoggedOut() {
    const res = await fetch(`${baseUrl}/users/auth/validate`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    if (!res.ok) {
        const returnUrl = encodeURIComponent(window.location.pathname);
        window.location.href = `${hostingPrefix}/login?next=${returnUrl}`;
    }
}
