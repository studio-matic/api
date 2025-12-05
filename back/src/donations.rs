use crate::{ApiResult, users::auth::validate};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;
use time::OffsetDateTime;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    get_donations,
    get_donation,
    post_donation,
    put_donation,
    delete_donation
))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Error, Debug)]
pub enum DonationError {
    #[error("Donation not found")]
    NotFound,
    #[error("Could not format")]
    FormatError(#[from] time::error::Format),
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for DonationError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
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

#[derive(Serialize, utoipa::ToSchema)]
struct DonationIdResponse {
    id: u64,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DonationRequest {
    coins: u64,
    income_eur: f64,
    co_op: String, // TODO: validate to be either "S4L" or "STUDIO-MATIC"
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
pub async fn get_donations(
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
            description = "Missing session_toke | Donation not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn get_donation(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;
    let donation: (u64, u64, OffsetDateTime, f64, String) = sqlx::query_as(
        "SELECT id, coins, donated_at, income_eur, co_op FROM donations WHERE id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state_pool.0)
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
pub async fn post_donation(
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

#[utoipa::path(
    put,
    path = "/donations/{id}",
    responses(
        (
            status = StatusCode::OK,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Missing session_toke | Donation not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn put_donation(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(req): Json<DonationRequest>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let res = sqlx::query(
        "UPDATE donations 
            SET 
                coins = ?,
                income_eur = ?,
                co_op =?
        WHERE id = ?",
    )
    .bind(req.coins)
    .bind(req.income_eur)
    .bind(req.co_op)
    .bind(id)
    .execute(&state_pool.0)
    .await
    .map_err(DonationError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(DonationError::NotFound.into())
    } else {
        Ok(StatusCode::OK)
    }
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
            description = "Missing session_token | Donation not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn delete_donation(
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
