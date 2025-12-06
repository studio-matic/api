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

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SupporterRequest {
    name: String,
    donation_id: u64,
}

pub mod delete;
pub mod get;
pub mod post;
pub mod put;
