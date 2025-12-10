async function loadTable({ url, selector, emptyText, columns }) {
    const tbody = document.querySelector(selector);
    tbody.innerHTML = `<tr><td colspan="2">Loading…</td></tr>`;

    try {
        const res = await fetch(url, {
            method: "GET",
            headers: { "Content-Type": "application/json" },
            credentials: "include"
        });

        if (!res.ok) {
            tbody.innerHTML = `<tr><td colspan="2">Failed to load data ❌</td></tr>`;
            return;
        }

        const data = await res.json();
        tbody.innerHTML = "";

        if (!data.length) {
            tbody.innerHTML = `<tr><td colspan="2">${emptyText}</td></tr>`;
            return;
        }

        data.forEach(item => {
            const tr = document.createElement("tr");
            tr.innerHTML = columns(item);
            tbody.appendChild(tr);
        });

    } catch (err) {
        console.error(err);
        tbody.innerHTML = `<tr><td colspan="2">Error connecting to backend ❌</td></tr>`;
    }
}

async function loadDbData() {
    await loadTable({
        url: `${baseUrl}/users`,
        selector: "#users tbody",
        emptyText: "No users yet",
        columns: ({ id, email }) => `
            <td>${email}</td>
            <td>
                <button class="delete-user" data-id="${id}">Delete</button>
            </td>
        `
    });
}

function enableForms() {
    document.querySelector("#users tbody").addEventListener("click", async (e) => {
        if (e.target.classList.contains("delete-user")) {
            const id = e.target.dataset.id;
            if (confirm("Are you sure you want to delete this user?")) {
                const res = await fetch(`${baseUrl}/users/${id}`, {
                    method: "DELETE",
                    headers: { "Content-Type": "application/json" },
                    credentials: "include"
                });

                if (res.ok) {
                    alert("User deleted ✅");
                    loadDbData();
                } else {
                    alert(await res.text());
                }
            }
        }
    });
}
