use crate::ApiResult;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};
use sqlx::MySqlPool;

use super::validate::{ValidationError, extract_session_token};

#[derive(utoipa::OpenApi)]
#[openapi(paths(signout))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    post,
    path = "/auth/signout",
    responses(
        (
            status = StatusCode::OK,
            description = "Logged out"
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
pub async fn signout(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let token = extract_session_token(headers)?;

    let _ = sqlx::query("DELETE FROM sessions WHERE token = ?")
        .bind(&token)
        .execute(&pool)
        .await
        .map_err(ValidationError::DatabaseError)?;

    #[cfg(debug_assertions)]
    let remove_cookie = "session_token=; Max-Age=0; Path=/; HttpOnly";
    #[cfg(not(debug_assertions))]
    let remove_cookie = "session_token=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=None";

    Ok((
        StatusCode::OK,
        AppendHeaders([(header::SET_COOKIE, remove_cookie)]),
    ))
}
