use crate::{ApiResult, auth::validate};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use thiserror::Error;

#[derive(utoipa::OpenApi)]
#[openapi(paths(
    get_supporters,
    get_supporter,
    post_supporter,
    put_supporter,
    delete_supporter
))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Error, Debug)]
pub enum SupporterError {
    #[error("Supporter not found")]
    NotFound,
    #[error("Could not format")]
    FormatError(#[from] time::error::Format),
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for SupporterError {
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
struct SupporterResponse {
    id: u64,
    name: String,
    donation_id: u64,
}

#[derive(Serialize, utoipa::ToSchema)]
struct SupporterIdResponse {
    id: u64,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SupporterRequest {
    name: String,
    donation_id: u64,
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
pub async fn get_supporters(
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
            description = "Missing session_token | Supporter not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn get_supporter(
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

#[utoipa::path(
    post,
    path = "/supporters",
    responses(
        (
            status = StatusCode::CREATED,
            body = SupporterIdResponse,
            description = "Successfully added supporter",
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
pub async fn post_supporter(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Json(req): Json<SupporterRequest>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let id = sqlx::query(
        "INSERT INTO supporters (name, donation_id)
        VALUES (?, ?)",
    )
    .bind(req.name)
    .bind(req.donation_id)
    .execute(&state_pool.0)
    .await
    .map_err(SupporterError::DatabaseError)?
    .last_insert_id();

    Ok((StatusCode::CREATED, Json(SupporterIdResponse { id })))
}

#[utoipa::path(
    put,
    path = "/supporters/{id}",
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully added supporter",
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Missing session_token | Supporter not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn put_supporter(
    state_pool: State<MySqlPool>,
    headers: HeaderMap,
    Path(id): Path<u64>,
    Json(req): Json<SupporterRequest>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate(state_pool.clone(), headers).await?;

    let res = sqlx::query(
        "UPDATE supporters 
            SET 
                name = ?,
                donation_id = ?
        WHERE id = ?",
    )
    .bind(req.name)
    .bind(req.donation_id)
    .bind(id)
    .execute(&state_pool.0)
    .await
    .map_err(SupporterError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(SupporterError::NotFound.into())
    } else {
        Ok(StatusCode::OK)
    }
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
            description = "Missing session_token | Supporter not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn delete_supporter(
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
