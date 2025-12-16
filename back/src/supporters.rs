use crate::ErrorResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{self, IntoResponse},
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

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[strum(prefix = "SUPPORTERS_")]
pub enum Error {
    #[error("Supporter not found")]
    NotFound,
    #[error("Could not query database")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> response::Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error = self.as_ref().to_string();
        let message = self.to_string();

        (status, Json(ErrorResponse { error, message })).into_response()
    }
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(as = supporters::Response)]
struct Response {
    id: u64,
    name: String,
    donation_id: u64,
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(as = supporters::Request)]
pub struct Request {
    name: String,
    donation_id: u64,
}

pub mod delete;
pub mod get;
pub mod post;
pub mod put;
