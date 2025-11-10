const baseUrl = window.location.hostname === "localhost"
    ? "http://localhost:3000"
    : "https://test-sm-website.fly.dev";

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

async function register() {
    const email = document.getElementById("email").value;
    const res = await fetch(`${baseUrl}/register`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email })
    });
    alert(await res.text());
}
