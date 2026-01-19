async function updateEmail() {
  const email = document.getElementById("email").value.trim();

  if (email === "") {
    alert("Email cannot be empty");
    return;
  }

  try {
    const res = await fetch(`${baseUrl}/users/me`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ email })
    });

    alert(await res.text());
  } catch (err) {
    console.error(err);
    alert("Failed to update email");
  }
}

async function updatePassword() {
  const password = document.getElementById("password").value;
  const confirmPassword = document.getElementById("confirmpassword").value;

  if (password !== confirmPassword) {
    alert("Passwords must match");
    return;
  }

  try {
    const res = await fetch(`${baseUrl}/users/me`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify({ password })
    });

    alert(await res.text());
  } catch (err) {
    console.error(err);
    alert("Failed to update password");
  }
}
