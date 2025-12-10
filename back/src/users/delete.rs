use crate::{
    ApiResult,
    users::{UserDataError, auth::validate},
};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(user))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    responses(
        (
            status = StatusCode::NO_CONTENT,
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
    )
)]
pub async fn user(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate::validate(state_pool.clone(), headers).await?;

    let res = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(id)
        .execute(&state_pool.0)
        .await
        .map_err(UserDataError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(UserDataError::NotFound.into())
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
