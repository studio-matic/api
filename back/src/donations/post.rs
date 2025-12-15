use crate::{
    ApiResult, AppState,
    donations::{self, DonationRequest},
    users::{UserRole, auth::validate},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

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
    State(AppState { pool }): State<AppState>,
    role: UserRole,
    Json(DonationRequest {
        coins,
        income_eur,
        co_op,
    }): Json<DonationRequest>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    let id = sqlx::query(
        "INSERT INTO donations (coins, income_eur, co_op)
        VALUES (?, ?, ?)",
    )
    .bind(coins)
    .bind(income_eur)
    .bind(co_op)
    .execute(&pool)
    .await
    .map_err(donations::Error::Database)?
    .last_insert_id();

    Ok((StatusCode::CREATED, Json(DonationIdResponse { id })))
}
