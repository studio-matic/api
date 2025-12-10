use crate::{
    ApiResult,
    users::{UserDataError, UserDataResponse, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(users, user))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<UserDataResponse>,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn users(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _ = validate::validate(state_pool.clone(), headers).await?;
    let users: Vec<(u64, String)> = sqlx::query_as("SELECT id, email FROM accounts")
        .fetch_all(&state_pool.0)
        .await
        .map_err(UserDataError::DatabaseError)?;

    let users = users
        .into_iter()
        .map(|(id, email)| Ok(UserDataResponse { id, email }))
        .collect::<ApiResult<Vec<_>>>()?;

    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (
            status = StatusCode::OK,
            body = UserDataResponse,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "User not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn user(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate::validate(state_pool.clone(), headers).await?;
    let user: (u64, String) = sqlx::query_as("SELECT id, email FROM accounts WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(&state_pool.0)
        .await
        .map_err(UserDataError::DatabaseError)?
        .ok_or(UserDataError::NotFound)?;

    let (id, email) = user;

    let user = UserDataResponse { id, email };

    Ok((StatusCode::OK, Json(user)))
}
