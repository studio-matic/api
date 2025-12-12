use crate::{ApiResult, users::UserRole};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(utoipa::OpenApi)]
#[openapi(paths(validate))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Cookies not found")]
    NoCookies,
    #[error("session_token cookie not found")]
    NoSessionToken,
    #[error("Invalid session token")]
    InvalidToken,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            _ => StatusCode::UNAUTHORIZED,
        };

        let msg = self.to_string();

        (status, Json(msg)).into_response()
    }
}

#[utoipa::path(
    get,
    path = "/users/auth/validate",
    responses(
        (
            status = StatusCode::OK,
            description = "Successful login",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Unsuccessful login",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR),
    ),
)]
pub async fn validate(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let session_token = extract_session_token(headers)?;

    if sqlx::query(
        "SELECT 1
            FROM sessions
            WHERE token = ?
            LIMIT 1",
    )
    .bind(session_token)
    .fetch_optional(&pool)
    .await
    .map_err(ValidationError::DatabaseError)?
    .is_some()
    {
        Ok(StatusCode::OK.into_response())
    } else {
        Err(ValidationError::InvalidToken.into())
    }
}

pub async fn validate_role(
    pool: &MySqlPool,
    headers: HeaderMap,
    role: UserRole,
) -> ApiResult<UserRole> {
    let session_token = extract_session_token(headers.clone())?;

    Ok(sqlx::query_scalar::<_, UserRole>(
        "SELECT accounts.role
                    FROM sessions JOIN accounts ON sessions.account_id = accounts.id
                        WHERE sessions.token = ?
                    LIMIT 1",
    )
    .bind(session_token)
    .fetch_optional(pool)
    .await
    .map_err(ValidationError::DatabaseError)?
    .ok_or(ValidationError::InvalidToken)
    .and_then(|r| {
        (r >= role)
            .then_some(r)
            .ok_or(ValidationError::InsufficientPermissions)
    })?)
}

pub fn extract_session_token(headers: HeaderMap) -> ApiResult<String> {
    let cookie_header = headers
        .get(header::COOKIE)
        .ok_or(ValidationError::NoCookies)?;
    let cookies = cookie_header.to_str().unwrap_or_default();
    let session_token = cookies
        .split(';')
        .find_map(|s| s.trim().strip_prefix("session_token="))
        .ok_or(ValidationError::NoSessionToken)?;
    Ok(session_token.to_owned())
}
