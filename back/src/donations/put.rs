use crate::{
    ApiResult, AppState,
    donations::{DonationError, DonationRequest},
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
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let res = sqlx::query(
        "UPDATE donations
            SET
                coins = ?,
                income_eur = ?,
                co_op =?
        WHERE id = ?",
    )
    .bind(coins)
    .bind(income_eur)
    .bind(co_op)
    .bind(id)
    .execute(&pool)
    .await
    .map_err(DonationError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(DonationError::NotFound.into())
    } else {
        Ok(StatusCode::OK)
    }
}
