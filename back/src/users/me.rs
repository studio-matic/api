use crate::{
    ApiResult,
    users::{UserDataError, UserDataResponse, auth::validate},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(me))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
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
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn me(State(pool): State<MySqlPool>, headers: HeaderMap) -> ApiResult<impl IntoResponse> {
    let session_token = validate::extract_session_token(headers)?;

    let user_data = sqlx::query_as::<_, UserDataResponse>(
        "SELECT accounts.id, accounts.email
            FROM sessions
            JOIN accounts ON accounts.id = sessions.account_id
            WHERE sessions.token = ?
            LIMIT 1",
    )
    .bind(session_token)
    .fetch_one(&pool)
    .await
    .map_err(UserDataError::DatabaseError)?;

    Ok((StatusCode::OK, Json(user_data)))
}
