use super::SignRequest;
use crate::{ApiResult, AppState, users::UserRole};
use argon2::{
    Argon2,
    password_hash::{self, PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(thiserror::Error, Debug)]
pub enum SignupError {
    #[error("Account already exists")]
    Conflict,
    #[error("Could not hash password: {0}")]
    PasswordHashError(#[from] password_hash::Error),
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for SignupError {
    fn into_response(self) -> Response {
        let status = match self {
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
            status = StatusCode::UNPROCESSABLE_ENTITY,
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
    State(AppState { pool }): State<AppState>,
    Json(SignRequest { email, password }): Json<SignRequest>,
) -> ApiResult<impl IntoResponse> {
    let hashed_password = Argon2::default()
        .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
        .map_err(SignupError::PasswordHashError)?
        .to_string();

    match sqlx::query("INSERT INTO accounts (email, password, role) VALUES (?, ?, ?)")
        .bind(&email)
        .bind(&hashed_password)
        .bind(UserRole::Editor)
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
