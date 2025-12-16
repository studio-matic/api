use crate::{ApiError, ApiResult, AppState, ErrorResponse, users::Role};
use axum::{
    Json,
    extract::{FromRequestParts, State},
    http::{HeaderMap, StatusCode, header, request::Parts},
    response::{IntoResponse, Response},
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(validate))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[strum(prefix = "VALIDATE_")]
pub enum Error {
    #[error("Cookies not found")]
    NoCookies,
    #[error("session_token cookie not found")]
    NoSessionToken,
    #[error("Invalid session token")]
    InvalidToken,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Could not query database")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::InsufficientPermissions => StatusCode::FORBIDDEN,
            _ => StatusCode::UNAUTHORIZED,
        };

        let error = self.as_ref().to_string();
        let message = self.to_string();

        (status, Json(ErrorResponse { error, message })).into_response()
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
    State(AppState { pool }): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    sqlx::query(
        "SELECT 1
            FROM sessions
            WHERE token = ?
            LIMIT 1",
    )
    .bind(extract_session_token(&headers)?)
    .fetch_optional(&pool)
    .await
    .map_err(Error::Database)?
    .is_some()
    .then_some(Ok(StatusCode::OK))
    .ok_or(Error::InvalidToken)?
}

pub async fn get_role(pool: &MySqlPool, headers: &HeaderMap) -> ApiResult<Role> {
    let token = extract_session_token(headers)?;

    Ok(sqlx::query_scalar::<_, Role>(
        "SELECT accounts.role
                    FROM sessions JOIN accounts ON sessions.account_id = accounts.id
                        WHERE sessions.token = ?
                    LIMIT 1",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
    .map_err(Error::Database)?
    .ok_or(Error::InvalidToken)?)
}

impl FromRequestParts<AppState> for Role {
    type Rejection = ApiError;

    async fn from_request_parts(
        Parts { headers, .. }: &mut Parts,
        AppState { pool }: &AppState,
    ) -> Result<Self, Self::Rejection> {
        get_role(pool, headers).await
    }
}

pub fn extract_session_token(headers: &HeaderMap) -> ApiResult<String> {
    let cookie_header = headers.get(header::COOKIE).ok_or(Error::NoCookies)?;
    let cookies = cookie_header.to_str().unwrap_or_default();
    let session_token = cookies
        .split(';')
        .find_map(|s| s.trim().strip_prefix("session_token="))
        .ok_or(Error::NoSessionToken)?;
    Ok(session_token.to_owned())
}
