use crate::{
    ApiResult,
    users::{UserDataError, UserRole, auth::validate},
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
        (
            status = StatusCode::FORBIDDEN,
            description = "Insufficient permissions",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn user(
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let role = validate::validate_role(&pool, headers, UserRole::Admin).await?;

    let _ = sqlx::query("SELECT 1 FROM accounts WHERE id = ? LIMIT 1")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(UserDataError::DatabaseError)?
        .ok_or(UserDataError::NotFound)?;

    Ok(
        sqlx::query("DELETE FROM accounts WHERE id = ? AND role < ?")
            .bind(id)
            .bind(u8::from(role))
            .execute(&pool)
            .await
            .map_err(UserDataError::DatabaseError)?
            .rows_affected()
            .ne(&0)
            .then_some(StatusCode::NO_CONTENT)
            .ok_or(validate::ValidationError::InsufficientPermissions)?,
    )
}
