const host = window.location.hostname;
let baseUrl, hostingPrefix;
if (host === "localhost" || host.startsWith("127.") || host === "[::1]") {
    baseUrl = `http://${host}:3000`;
    hostingPrefix = ""
} else {
    baseUrl = "https://test-sm-website.fly.dev";
    hostingPrefix = "/test-website"
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
setInterval(checkBackend, 5000)

async function welc() {
    const el = document.getElementById("welc");
    const res = await fetch(`${baseUrl}/auth/validate`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    if (res.ok) {
        const data = await res.json();
        el.innerText = el.innerText + " " + data.email;
        alert(data.message);
    }
}

async function signup() {
    const email = document.getElementById("email").value;
    const password = document.getElementById("password").value;
    const res = await fetch(`${baseUrl}/auth/signup`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
        credentials: "include"
    });
    alert(await res.text());
}

async function signin() {
    const email = document.getElementById("email").value;
    const password = document.getElementById("password").value;
    const res = await fetch(`${baseUrl}/auth/signin`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
        credentials: "include"
    });
    if (res.ok) {
        const params = new URLSearchParams(window.location.search);
        const nextPage = params.get('next') || `${hostingPrefix}/`;
        window.location.href = nextPage;
    } else {
        alert(await res.text());
    }
}

async function cookiesignin() {
    const res = await fetch(`${baseUrl}/auth/validate`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    alert(await res.text());
}

async function redirLoggedOut() {
    const res = await fetch(`${baseUrl}/auth/validate`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    if (!res.ok) {
        const returnUrl = encodeURIComponent(window.location.pathname);
        window.location.href = `${hostingPrefix}/login?next=${returnUrl}`;
    }
}
