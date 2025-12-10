async function signup() {
    const email = document.getElementById("email").value;
    const password = document.getElementById("password").value;
    const res = await fetch(`${baseUrl}/users/auth/signup`, {
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
    const res = await fetch(`${baseUrl}/users/auth/signin`, {
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
    const res = await fetch(`${baseUrl}/users/auth/signout`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    updateAuthUI();
    alert(await res.text());
}

async function cookiesignin() {
    const res = await fetch(`${baseUrl}/users/me`, {
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
    const res = await fetch(`${baseUrl}/users/auth/validate`, {
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
