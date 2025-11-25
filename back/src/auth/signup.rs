use super::{SESSION_TOKEN_MAX_AGE, SignRequest, generate_session_token};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};
use emval::ValidationError;
use sqlx::MySqlPool;
use tokio::task;

#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    post,
    path = "/signup",
    responses(
        (
            status = StatusCode::CREATED, description = "Successful signup",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            description = "Internal server error | Unsuccessful signup, but could not save session token",
        ),
        (
            status = StatusCode::CONFLICT,
            description = "Unsuccessful signup: Account already exists",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Unsuccessful signup: Invalid email",
        ),
    ),
)]
pub async fn signup(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let email = match task::spawn_blocking(|| emval::validate_email(req.email))
        .await
        .unwrap()
    {
        Ok(v) => v,
        Err(ValidationError::SyntaxError(e)) | Err(ValidationError::ValueError(e)) => {
            return (StatusCode::BAD_REQUEST, e).into_response();
        }
    }
    .normalized;

    let hashed_password = Argon2::default()
        .hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    let account_result = sqlx::query("INSERT INTO accounts (email, password) VALUES (?, ?)")
        .bind(&email)
        .bind(&hashed_password)
        .execute(&pool)
        .await;

    let token = generate_session_token();

    match account_result {
        Ok(_) => {
            if let Err(e) = sqlx::query(
                "INSERT INTO sessions (token, account_id, expires_at)
                VALUES (
                    ?,
                    (SELECT id FROM accounts WHERE email = ?),
                    NOW() + INTERVAL ? SECOND
                )",
            )
            .bind(&token)
            .bind(&email)
            .bind(SESSION_TOKEN_MAX_AGE.as_secs())
            .execute(&pool)
            .await
            {
                eprintln!("{e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Unsuccessful signup, but could not save session token"),
                )
                    .into_response();
            }
            (
                StatusCode::CREATED,
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
                Json("Successful signup"),
            )
                .into_response()
        }
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => (
            StatusCode::CONFLICT,
            Json("Unsuccessful signup: Account already exists"),
        )
            .into_response(),
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
