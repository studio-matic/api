use crate::{
    ApiResult, AppState,
    donations::{self, DonationRequest},
    users::{UserRole, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    put,
    path = "/donations/{id}",
    responses(
        (
            status = StatusCode::OK,
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
    State(AppState { pool }): State<AppState>,
    role: UserRole,
    Path(id): Path<u64>,
    Json(DonationRequest {
        coins,
        income_eur,
        co_op,
    }): Json<DonationRequest>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    Ok(
        sqlx::query("UPDATE donations SET coins = ?, income_eur = ?, co_op = ? WHERE id = ?")
            .bind(coins)
            .bind(income_eur)
            .bind(co_op)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(donations::Error::Database)?
            .rows_affected()
            .ne(&0)
            .then_some(StatusCode::OK)
            .ok_or(donations::Error::NotFound)?,
    )
}
