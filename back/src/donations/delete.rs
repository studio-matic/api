use crate::{ApiResult, donations::DonationError, users::auth::validate};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    delete,
    path = "/donations/{id}",
    responses(
        (
            status = StatusCode::NO_CONTENT,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Donation not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn donation(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let res = sqlx::query("DELETE FROM donations WHERE id = ?")
        .bind(id)
        .execute(&state_pool.0)
        .await
        .map_err(DonationError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(DonationError::NotFound.into())
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
