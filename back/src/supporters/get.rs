use crate::{
    ApiResult,
    supporters::{SupporterError, SupporterResponse},
    users::auth::validate,
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(supporters, supporter))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    get,
    path = "/supporters",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<SupporterResponse>,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn supporters(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;
    let supporters: Vec<(u64, String, u64)> =
        sqlx::query_as("SELECT id, name, donation_id FROM supporters")
            .fetch_all(&state_pool.0)
            .await
            .map_err(SupporterError::DatabaseError)?;

    let supporters = supporters
        .into_iter()
        .map(|(a, b, c)| {
            Ok(SupporterResponse {
                id: a,
                name: b,
                donation_id: c,
            })
        })
        .collect::<ApiResult<Vec<_>>>()?;

    Ok((StatusCode::OK, Json(supporters)))
}

#[utoipa::path(
    get,
    path = "/supporters/{id}",
    responses(
        (
            status = StatusCode::OK,
            body = SupporterResponse,
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
    ),
)]
pub async fn supporter(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;
    let supporter: (u64, String, u64) = sqlx::query_as(
        "SELECT id, name, donation_id FROM supporters WHERE supporters.id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state_pool.0)
    .await
    .map_err(SupporterError::DatabaseError)?
    .ok_or(SupporterError::NotFound)?;

    let (id, name, donation_id) = supporter;

    let supporter = SupporterResponse {
        id,
        name,
        donation_id,
    };

    Ok((StatusCode::OK, Json(supporter)))
}
