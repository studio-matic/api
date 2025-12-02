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
setInterval(checkBackend, 10000)

async function welc() {
    const el = document.getElementById("welc");
    const res = await fetch(`${baseUrl}/me`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    if (res.ok) {
        const data = await res.json();
        el.innerText = el.innerText + " " + data.email;
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
    if (res.ok) {
        signin();
    } else {
        alert(await res.text());
    }
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

async function signout() {
    const res = await fetch(`${baseUrl}/auth/signout`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    updateAuthUI();
    alert(await res.text());
}

async function cookiesignin() {
    const res = await fetch(`${baseUrl}/me`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    alert(await res.text());
}

async function updateAuthUI() {
    const email = document.getElementById("email");
    const password = document.getElementById("password");
    const signup = document.getElementById("signup");
    const signin = document.getElementById("signin");
    const signout = document.getElementById("signout");
    const res = await fetch(`${baseUrl}/auth/validate`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    if (res.ok) {
        email.hidden = true;
        password.hidden = true;
        signup.hidden = true;
        signin.hidden = true;
        signout.hidden = false;
    } else {
        email.hidden = false;
        password.hidden = false;
        signup.hidden = false;
        signin.hidden = false;
        signout.hidden = true;
    }
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

function prettyDate(isoString) {
    const d = new Date(isoString);
    return d.toLocaleDateString("en-GB", {
        year: "numeric",
        month: "short",
        day: "2-digit",
    });
}

async function loadTable({ url, selector, emptyText, columns }) {
    const tbody = document.querySelector(selector);
    tbody.innerHTML = `<tr><td colspan="4">Loading…</td></tr>`;

    try {
        const res = await fetch(url, {
            method: "GET",
            headers: { "Content-Type": "application/json" },
            credentials: "include"
        });

        if (!res.ok) {
            tbody.innerHTML = `<tr><td colspan="4">Failed to load data ❌</td></tr>`;
            return;
        }

        const data = await res.json();
        tbody.innerHTML = "";

        if (!data.length) {
            tbody.innerHTML = `<tr><td colspan="4">${emptyText}</td></tr>`;
            return;
        }

        data.forEach(item => {
            const tr = document.createElement("tr");
            tr.innerHTML = columns(item);
            tbody.appendChild(tr);
        });

    } catch (err) {
        console.error(err);
        tbody.innerHTML = `<tr><td colspan="4">Error connecting to backend ❌</td></tr>`;
    }
}

async function loadDbData() {
    await loadTable({
        url: `${baseUrl}/donations`,
        selector: "#donations tbody",
        emptyText: "No donations yet",
        columns: ({ coins, donated_at, income_eur, co_op }) => `
            <td>${coins}</td>
            <td>${prettyDate(donated_at)}</td>
            <td>${income_eur.toFixed(2)}</td>
            <td>${co_op}</td>
        `
    });

    await loadTable({
        url: `${baseUrl}/supporters`,
        selector: "#supporters tbody",
        emptyText: "No supporters yet",
        columns: ({ name, supported_at, income_eur, co_op }) => `
            <td>${name}</td>
            <td>${prettyDate(supported_at)}</td>
            <td>${income_eur.toFixed(2)}</td>
            <td>${co_op}</td>
        `
    });
}

async function enableForms() {
    document.getElementById("add-donation-form").addEventListener("submit", async (e) => {
        e.preventDefault();

        const coins = parseInt(document.getElementById("donation-coins").value, 10);
        const income_eur = parseFloat(document.getElementById("donation-income").value);
        const co_op = "STUDIO-MATIC";
        const statusEl = document.getElementById("add-donation-status");

        try {
            const res = await fetch(`${baseUrl}/donations`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                credentials: "include",
                body: JSON.stringify({ coins, income_eur, co_op })
            });

            if (res.ok) {
                statusEl.innerText = "Donation added ✅";
                document.getElementById("add-donation-form").reset();
                loadDbData();

                setTimeout(() => {
                    statusEl.innerText = "";
                }, 5000);
            } else {
                const text = await res.text();
                statusEl.innerText = `Failed ❌: ${text}`;
            }
        } catch (err) {
            console.error(err);
            statusEl.innerText = "Error connecting to backend ❌";
        }
    });
}
