use crate::{
    ApiResult,
    donations::{DonationError, DonationRequest},
    users::auth::validate,
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
struct DonationIdResponse {
    id: u64,
}

#[utoipa::path(
    post,
    path = "/donations",
    responses(
        (
            status = StatusCode::CREATED,
            body = DonationIdResponse,
            description = "Successfully added donation",
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
    Json(req): Json<DonationRequest>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let id = sqlx::query(
        "INSERT INTO donations (coins, income_eur, co_op)
        VALUES (?, ?, ?)",
    )
    .bind(req.coins)
    .bind(req.income_eur)
    .bind(req.co_op)
    .execute(&state_pool.0)
    .await
    .map_err(DonationError::DatabaseError)?
    .last_insert_id();

    Ok((StatusCode::CREATED, Json(DonationIdResponse { id })))
}
