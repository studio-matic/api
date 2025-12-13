use crate::{
    ApiResult, AppState,
    donations::{DonationError, DonationResponse},
    users::{UserRole, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use time::OffsetDateTime;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donations, donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}
#[utoipa::path(
    get,
    path = "/donations",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<DonationResponse>,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn donations(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let donations: Vec<(u64, u64, OffsetDateTime, f64, String)> =
        sqlx::query_as("SELECT id, coins, donated_at, income_eur, co_op FROM donations")
            .fetch_all(&pool)
            .await
            .map_err(DonationError::DatabaseError)?;

    let donations = donations
        .into_iter()
        .map(|(a, b, c, d, e)| {
            Ok(DonationResponse {
                id: a,
                coins: b,
                donated_at: c
                    .to_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .map_err(DonationError::FormatError)?,
                income_eur: d,
                co_op: e,
            })
        })
        .collect::<ApiResult<Vec<_>>>()?;

    Ok((StatusCode::OK, Json(donations)))
}

#[utoipa::path(
    get,
    path = "/donations/{id}",
    responses(
        (
            status = StatusCode::OK,
            body = DonationResponse,
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
    ),
)]
pub async fn donation(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let donation: (u64, u64, OffsetDateTime, f64, String) = sqlx::query_as(
        "SELECT id, coins, donated_at, income_eur, co_op FROM donations WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(DonationError::DatabaseError)?
    .ok_or(DonationError::NotFound)?;

    let (id, coins, donated_at, income_eur, co_op) = donation;

    let donations = DonationResponse {
        id,
        coins,
        donated_at: donated_at
            .to_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(DonationError::FormatError)?,
        income_eur,
        co_op,
    };

    Ok((StatusCode::OK, Json(donations)))
}
