use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(utoipa::OpenApi)]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    let mut api = ApiDoc::openapi();
    api.merge(post::openapi());
    api.merge(get::openapi());
    api.merge(put::openapi());
    api.merge(delete::openapi());
    api
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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct DonationRequest {
    coins: u64,
    income_eur: f64,
    co_op: String, // TODO: validate to be either "S4L" or "STUDIO-MATIC"
}

pub mod delete;
pub mod get;
pub mod post;
pub mod put;
