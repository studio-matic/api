use crate::ErrorResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{self, IntoResponse},
};
use serde::{Deserialize, Serialize};
use sqlx::Type;

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

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[strum(prefix = "DONATIONS_")]
pub enum Error {
    #[error("Donation not found")]
    NotFound,
    #[error("Could not format time")]
    TimeFormat(#[from] time::error::Format),
    #[error("Could not query database")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> response::Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::TimeFormat(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error = self.as_ref().to_string();
        let message = self.to_string();

        (status, Json(ErrorResponse { error, message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(as = donations::Response)]
struct Response {
    id: u64,
    coins: u64,
    donated_at: String,
    income_eur: f64,
    co_op: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(as = donations::Request)]
pub struct Request {
    coins: u64,
    income_eur: f64,
    co_op: CoOp,
}

#[derive(Deserialize, Type, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoOp {
    S4l,
    StudioMatic,
}

pub mod delete;
pub mod get;
pub mod post;
pub mod put;
