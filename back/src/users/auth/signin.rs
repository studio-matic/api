use crate::{
    ApiError, ApiResult, AppState, ErrorResponse,
    users::{
        auth::{SESSION_TOKEN_MAX_AGE, generate_session_token},
        email::EmailAddress,
    },
};
use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse, Response},
};
use axum_extra::extract::WithRejection as Rejectable;
use serde::Deserialize;

#[derive(utoipa::OpenApi)]
#[openapi(paths(signin))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SigninRequest {
    email: EmailAddress,
    password: String,
}

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[strum(prefix = "SIGNIN_")]
pub enum Error {
    #[error("Password incorrect")]
    IncorrectPassword,
    #[error("Account not found")]
    AccountNotFound,
    #[error("Could not hash password")]
    PasswordHash(#[from] password_hash::Error),
    #[error("Could not query database")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::IncorrectPassword => StatusCode::UNAUTHORIZED,
            Self::AccountNotFound => StatusCode::NOT_FOUND,
            Self::PasswordHash(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error = self.as_ref().to_string();
        let message = self.to_string();

        (status, Json(ErrorResponse { error, message })).into_response()
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
            status = StatusCode::UNPROCESSABLE_ENTITY,
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
    State(AppState { pool }): State<AppState>,
    Rejectable(Json(SigninRequest { email, password }), _): Rejectable<
        Json<SigninRequest>,
        ApiError,
    >,
) -> ApiResult<impl IntoResponse> {
    let (id, hashed_password): (u64, String) =
        sqlx::query_as("SELECT id, password FROM accounts WHERE email = ? LIMIT 1")
            .bind(&email)
            .fetch_optional(&pool)
            .await
            .map_err(Error::Database)?
            .ok_or(Error::AccountNotFound)?;

    Argon2::default()
        .verify_password(
            password.as_bytes(),
            &PasswordHash::new(&hashed_password).map_err(Error::PasswordHash)?,
        )
        .map_err(|e| match e {
            password_hash::Error::Password => Error::IncorrectPassword,
            e => Error::PasswordHash(e),
        })?;

    let token = generate_session_token();

    let _ = sqlx::query(
            "INSERT INTO sessions (token, account_id, expires_at) VALUES (?, ?, NOW() + INTERVAL ? SECOND)",
        )
        .bind(&token)
        .bind(id)
        .bind(SESSION_TOKEN_MAX_AGE.as_secs())
        .execute(&pool)
        .await
        .map_err(Error::Database)?;

    Ok(AppendHeaders([(
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
    )]))
}
