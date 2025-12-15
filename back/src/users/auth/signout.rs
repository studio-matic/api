use crate::{
    ApiResult, AppState,
    users::auth::validate::{self, extract_session_token},
};
use axum::{
    extract::State,
    http::{HeaderMap, header},
    response::{AppendHeaders, IntoResponse},
};
#[derive(utoipa::OpenApi)]
#[openapi(paths(signout))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    delete,
    path = "/users/auth/signout",
    responses(
        (
            status = StatusCode::OK,
            description = "Logged out"
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn signout(
    State(AppState { pool }): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _ = sqlx::query("DELETE FROM sessions WHERE token = ? LIMIT 1")
        .bind(&extract_session_token(&headers)?)
        .execute(&pool)
        .await
        .map_err(validate::Error::Database)?;

    #[cfg(debug_assertions)]
    let remove_cookie = "session_token=; Max-Age=0; Path=/; HttpOnly";
    #[cfg(not(debug_assertions))]
    let remove_cookie = "session_token=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=None";

    Ok(AppendHeaders([(header::SET_COOKIE, remove_cookie)]))
}
