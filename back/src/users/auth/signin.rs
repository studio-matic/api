use crate::ApiResult;

use super::{SESSION_TOKEN_MAX_AGE, SignRequest, generate_session_token};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use emval::ValidationError as EmailValidationError;
use sqlx::MySqlPool;
use tokio::task;

#[derive(utoipa::OpenApi)]
#[openapi(paths(signin))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(thiserror::Error, Debug)]
pub enum SigninError {
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    #[error("Password incorrect")]
    IncorrectPassword,
    #[error("Account not found")]
    AccountNotFound,
    #[error("Could not save session token: {0}")]
    SessionError(String),
    #[error("Could not hash password: {0}")]
    PasswordHashError(String),
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for SigninError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InvalidEmail(_) => StatusCode::BAD_REQUEST,
            Self::IncorrectPassword => StatusCode::UNAUTHORIZED,
            Self::AccountNotFound => StatusCode::NOT_FOUND,
            Self::SessionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let msg = self.to_string();

        (status, Json(msg)).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/users/auth/signin",
    responses(
        (
            status = StatusCode::OK,
            description = "Successful signin"
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Invalid email",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Password incorrect",
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Account not found",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
)]
pub async fn signin(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> ApiResult<impl IntoResponse> {
    let email = task::spawn_blocking(|| emval::validate_email(req.email))
        .await
        .expect("Unable to join email validation thread")
        .map_err(|e| {
            SigninError::InvalidEmail(match e {
                EmailValidationError::SyntaxError(e) | EmailValidationError::ValueError(e) => e,
            })
        })?
        .normalized;

    let (id, hashed_password): (u64, String) =
        sqlx::query_as("SELECT id, password FROM accounts WHERE email = ? LIMIT 1")
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .map_err(SigninError::DatabaseError)?
            .ok_or(SigninError::AccountNotFound)?;

    if Argon2::default()
        .verify_password(
            req.password.as_bytes(),
            &PasswordHash::new(&hashed_password)
                .map_err(|e| SigninError::PasswordHashError(e.to_string()))?,
        )
        .is_ok()
    {
        let token = generate_session_token();

        let _ = sqlx::query(
            "INSERT INTO sessions (token, account_id, expires_at)
                VALUES (
                    ?,
                    ?,
                    NOW() + INTERVAL ? SECOND
                )",
        )
        .bind(&token)
        .bind(id)
        .bind(SESSION_TOKEN_MAX_AGE.as_secs())
        .execute(&pool)
        .await
        .map_err(|e| SigninError::SessionError(e.to_string()))?;

        Ok((
            StatusCode::OK,
            AppendHeaders([(
                header::SET_COOKIE,
                #[cfg(debug_assertions)]
                format!(
                    "session_token={token}; Max-Age={}; Path=/; HttpOnly",
                    SESSION_TOKEN_MAX_AGE.as_secs()
                ),
                #[cfg(not(debug_assertions))]
                format!(
                    "session_token={token}; Max-Age={}; Path=/; HttpOnly; Secure; SameSite=None",
                    SESSION_TOKEN_MAX_AGE.as_secs()
                ),
            )]),
            Json("Successful signin"),
        )
            .into_response())
    } else {
        Err(SigninError::IncorrectPassword.into())
    }
}
