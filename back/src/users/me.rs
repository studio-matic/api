use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::MySqlPool;

use crate::{
    ApiResult,
    users::auth::validate::{self, ValidationError},
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(me))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
struct UserDataResponse {
    email: String,
    id: u64,
}

#[utoipa::path(
    get,
    path = "/users/me",
    responses(
        (
            status = StatusCode::OK,
            body = UserDataResponse
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Missing session_token",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn me(State(pool): State<MySqlPool>, headers: HeaderMap) -> ApiResult<impl IntoResponse> {
    let session_token = validate::extract_session_token(headers)?;

    match sqlx::query_as(
        "SELECT accounts.id, accounts.email
            FROM sessions
            JOIN accounts ON accounts.id = sessions.account_id
            WHERE sessions.token = ?
            LIMIT 1",
    )
    .bind(session_token)
    .fetch_optional(&pool)
    .await
    .map_err(ValidationError::DatabaseError)?
    {
        Some(v) => {
            let (id, email): (u64, String) = v;
            Ok((StatusCode::OK, Json(UserDataResponse { id, email })))
        }
        None => Err(ValidationError::InvalidToken.into()),
    }
}
