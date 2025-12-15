use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Donation not found")]
    NotFound,
    #[error("Could not format")]
    Format(#[from] time::error::Format),
    #[error("Could not query database")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Format(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
