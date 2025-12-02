use crate::{ApiResult, auth::validate};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donations, add_donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Error, Debug)]
pub enum DonationError {
    #[error("Could not format")]
    FormatError(#[from] time::error::Format),
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for DonationError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::FormatError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let msg = self.to_string();

        (status, Json(msg)).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
struct DonationResponse {
    id: u64,
    coins: u64,
    donated_at: String,
    income_eur: f64,
    co_op: String,
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
            status = StatusCode::NOT_FOUND,
            description = "Missing session_token",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn donations(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;
    let donations: Vec<(u64, u64, OffsetDateTime, f64, String)> =
        sqlx::query_as("SELECT id, coins, donated_at, income_eur, co_op FROM donations")
            .fetch_all(&state_pool.0)
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DonationRequest {
    coins: u64,
    income_eur: f64,
    co_op: String, // TODO: validate to be either "S4L" or "STUDIO-MATIC"
}

#[utoipa::path(
    post,
    path = "/donations",
    responses(
        (
            status = StatusCode::CREATED,
            description = "Successfully added donation",
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Missing session_token",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn add_donation(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Json(req): Json<DonationRequest>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let _ = sqlx::query(
        "INSERT INTO donations (coins, income_eur, co_op)
        VALUES (?, ?, ?)",
    )
    .bind(req.coins)
    .bind(req.income_eur)
    .bind(req.co_op)
    .execute(&state_pool.0)
    .await
    .map_err(DonationError::DatabaseError)?;

    Ok(StatusCode::CREATED)
}
