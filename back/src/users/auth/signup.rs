use super::SignRequest;
use crate::ApiResult;
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use emval::ValidationError as EmailValidationError;
use sqlx::MySqlPool;
use tokio::task;

#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(thiserror::Error, Debug)]
pub enum SignupError {
    #[error("Invalid email: {0}")]
    InvalidEmail(String),
    #[error("Account already exists")]
    Conflict,
    #[error("Could not hash password: {0}")]
    PasswordHashError(String),
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InvalidEmail(_) => StatusCode::BAD_REQUEST,
            Self::Conflict => StatusCode::CONFLICT,
            Self::PasswordHashError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let msg = self.to_string();

        (status, Json(msg)).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/users/auth/signup",
    responses(
        (
            status = StatusCode::CREATED,
            description = "Successful signup",
        ),
        (
            status = StatusCode::BAD_REQUEST,
            description = "Invalid email",
        ),
        (
            status = StatusCode::CONFLICT,
            description = "Account already exists",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ),
)]
pub async fn signup(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> ApiResult<impl IntoResponse> {
    let email = task::spawn_blocking(|| emval::validate_email(req.email))
        .await
        .expect("Unable to join email validation thread")
        .map_err(|e| {
            SignupError::InvalidEmail(match e {
                EmailValidationError::SyntaxError(e) | EmailValidationError::ValueError(e) => e,
            })
        })?
        .normalized;

    let hashed_password = Argon2::default()
        .hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(|e| SignupError::PasswordHashError(e.to_string()))?
        .to_string();

    match sqlx::query("INSERT INTO accounts (email, password) VALUES (?, ?)")
        .bind(&email)
        .bind(&hashed_password)
        .execute(&pool)
        .await
    {
        Ok(_) => Ok(StatusCode::CREATED.into_response()),
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            Err(SignupError::Conflict.into())
        }
        Err(e) => Err(SignupError::DatabaseError(e).into()),
    }
}
