use crate::{
    ApiResult, AppState,
    users::{UserRole, email::EmailAddress},
};
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
use serde::Deserialize;

#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SignupRequest {
    email: EmailAddress,
    password: String,
    invite: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid invite")]
    ExpiredInvite,
    #[error("Invite not found")]
    InviteNotFound,
    #[error("Account already exists")]
    Conflict,
    #[error("Could not hash password: {0}")]
    PasswordHash(#[from] password_hash::Error),
    #[error("Could not query database")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::InviteNotFound => StatusCode::NOT_FOUND,
            Self::ExpiredInvite => StatusCode::GONE,
            Self::Conflict => StatusCode::CONFLICT,
            Self::PasswordHash(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
    Json(SignupRequest {
        email,
        password,
        invite,
    }): Json<SignupRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut transaction = pool.begin().await.map_err(Error::Database)?;

    let (id, role): (u64, UserRole) =
        sqlx::query_as("SELECT id, role FROM invites WHERE code = ? LIMIT 1")
            .bind(&invite)
            .fetch_optional(&pool)
            .await
            .map_err(Error::Database)?
            .ok_or(Error::InviteNotFound)?;

    if sqlx::query(
        "UPDATE invites SET expires_at = NOW() WHERE id = ? AND expires_at > NOW() LIMIT 1",
    )
    .bind(id)
    .execute(&mut *transaction)
    .await
    .map_err(Error::Database)?
    .rows_affected()
    .eq(&0)
    {
        Err(Error::ExpiredInvite)?
    }

    match sqlx::query("INSERT INTO accounts (email, password, role) VALUES (?, ?, ?)")
        .bind(&email)
        .bind(
            Argon2::default()
                .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
                .map_err(Error::PasswordHash)?
                .to_string(),
        )
        .bind(role)
        .execute(&mut *transaction)
        .await
    {
        Ok(_) => {
            transaction.commit().await.map_err(Error::Database)?;

            Ok(StatusCode::CREATED)
        }
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(Error::Conflict)?,
        Err(e) => Err(Error::Database(e))?,
    }
}
