async function redirUnauthorized() {
    const res = await fetch(`${baseUrl}/users/me`, {
        method: "GET",
        headers: { "Content-Type": "application/json" },
        credentials: "include"
    });
    const { role } = await res.json();
    if (!res.ok || role !== "superadmin") {
        alert(`Must be a superadmin`)
        const returnUrl = encodeURIComponent(window.location.pathname);
        window.location.href = `${hostingPrefix}/login?next=${returnUrl}`;
    }
}

async function invite() {
    const roleInput = document.getElementById('role');
    const role = roleInput.value.trim();
    const invitesTableBody = document.querySelector('#invites tbody');

    if (!role) {
        alert("Please enter a role");
        return;
    }

    try {
        const res = await fetch(`${baseUrl}/users/auth/invite`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ role }),
            credentials: "include"
        });

        if (res.ok) {
            alert(`Invite for role "${role}" created ✅`);

            const { code } = await res.json()

            const row = document.createElement('tr');
            const codeCell = document.createElement('td');
            codeCell.textContent = code;
            const roleCell = document.createElement('td');
            roleCell.textContent = role;
            row.appendChild(codeCell);
            row.appendChild(roleCell);
            invitesTableBody.appendChild(row);
            roleInput.value = '';
        } else {
            alert(await res.text());
        }
    } catch (err) {
        console.error(err);
        statusDiv.textContent = "Failed to send invite ❌";
    }
}
