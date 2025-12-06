use crate::{ApiResult, supporters::SupporterError, users::auth::validate};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(supporter))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    delete,
    path = "/supporters/{id}",
    responses(
        (
            status = StatusCode::NO_CONTENT,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Supporter not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn supporter(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let res = sqlx::query("DELETE FROM supporters WHERE id = ?")
        .bind(id)
        .execute(&state_pool.0)
        .await
        .map_err(SupporterError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(SupporterError::NotFound.into())
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
