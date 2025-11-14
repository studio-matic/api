use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
use std::time::Duration;

const SESSION_TOKEN_MAX_AGE: Duration = Duration::from_hours(1);

#[derive(Deserialize)]
pub struct SignRequest {
    email: String,
    password: String,
}

fn generate_session_token() -> String {
    (0..64)
        .map(|_| {
            const BASE64_ALPHABET: &[u8] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let idx = rand::random_range(0..BASE64_ALPHABET.len());
            BASE64_ALPHABET[idx] as char
        })
        .collect::<String>()
}

pub async fn signup(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let token = generate_session_token();

    let hashed_password = Argon2::default()
        .hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    let account_result = sqlx::query("INSERT INTO accounts (email, password) VALUES (?, ?)")
        .bind(&req.email)
        .bind(&hashed_password)
        .execute(&pool)
        .await;

    match account_result {
        Ok(_) => {
            if let Err(e) = sqlx::query("INSERT INTO sessions (token, email) VALUES (?, ?)")
                .bind(&token)
                .bind(&req.email)
                .execute(&pool)
                .await
            {
                eprintln!("{e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Account created, but could not save session token"),
                )
                    .into_response();
            }
            (
                StatusCode::OK,
                AppendHeaders([(
                    header::SET_COOKIE,
                    #[cfg(debug_assertions)]
                    format!(
                        "session_token={token}; Max-Age={}; HttpOnly",
                        SESSION_TOKEN_MAX_AGE.as_secs()
                    ),
                    #[cfg(not(debug_assertions))]
                    format!(
                        "session_token={token}; Max-Age={}; HttpOnly; Secure; SameSite=None",
                        SESSION_TOKEN_MAX_AGE.as_secs()
                    ),
                )]),
                Json("Account created successfully"),
            )
                .into_response()
        }
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            (StatusCode::CONFLICT, Json("Account already exists")).into_response()
        }
        Err(e) => {
            eprintln!("{e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error"),
            )
                .into_response()
        }
    }
}

pub async fn signin(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let hashed_password = if let Some(v) =
        sqlx::query("SELECT (password) FROM accounts WHERE email = ?")
            .bind(&req.email)
            .fetch_optional(&pool)
            .await
            .unwrap()
    {
        v
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json("Unsucccesful login: account not found"),
        )
            .into_response();
    }
    .get::<String, _>(0);

    let token = generate_session_token();

    if Argon2::default()
        .verify_password(
            req.password.as_bytes(),
            &PasswordHash::new(&hashed_password).unwrap(),
        )
        .is_ok()
    {
        if let Err(e) = sqlx::query("INSERT INTO sessions (token, email) VALUES (?, ?)")
            .bind(&token)
            .bind(&req.email)
            .execute(&pool)
            .await
        {
            eprintln!("{e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Succcesful login, but could not save session token"),
            )
                .into_response()
        } else {
            (
                StatusCode::OK,
                AppendHeaders([(
                    header::SET_COOKIE,
                    #[cfg(debug_assertions)]
                    format!(
                        "session_token={token}; Max-Age={}; HttpOnly",
                        SESSION_TOKEN_MAX_AGE.as_secs()
                    ),
                    #[cfg(not(debug_assertions))]
                    format!(
                        "session_token={token}; Max-Age={}; HttpOnly; Secure; SameSite=None",
                        SESSION_TOKEN_MAX_AGE.as_secs()
                    ),
                )]),
                Json("Succcesful login"),
            )
                .into_response()
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json("Unsucccesful login: password incorrect"),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct ValidationResponse {
    email: String,
    message: &'static str,
}

pub async fn validate(State(pool): State<MySqlPool>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        let cookies = cookie_header.to_str().unwrap_or_default();
        let session_token = if let Some(v) = cookies
            .split(';')
            .find_map(|s| s.trim().strip_prefix("session_token="))
        {
            v
        } else {
            return (
                StatusCode::UNAUTHORIZED,
                Json("Unsucccesful login: session_token cookie not found"),
            )
                .into_response();
        };
        match sqlx::query("SELECT (email) FROM sessions WHERE token = ?")
            .bind(session_token)
            .fetch_optional(&pool)
            .await
            .unwrap()
        {
            Some(email) => (
                StatusCode::OK,
                Json(ValidationResponse {
                    email: email.get::<String, _>(0),
                    message: "Succcesful login",
                }),
            )
                .into_response(),
            None => (
                StatusCode::UNAUTHORIZED,
                Json("Unsucccesful login: session token not found"),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json("Unsucccesful login: cookies not found"),
        )
            .into_response()
    }
}

pub async fn cleanup_expired_sessions(pool: MySqlPool) {
    let mut interval = tokio::time::interval(SESSION_TOKEN_MAX_AGE);
    loop {
        interval.tick().await;

        match sqlx::query("DELETE FROM sessions WHERE created_at < NOW() - INTERVAL ? SECOND")
            .bind(SESSION_TOKEN_MAX_AGE.as_secs())
            .execute(&pool)
            .await
        {
            Ok(res) => println!("Deleted {} expired sessions", res.rows_affected()),
            Err(e) => eprintln!("Failed to cleanup expired sessions: {e}"),
        }
    }
}
