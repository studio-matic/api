use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

pub mod auth;
pub mod me;

#[derive(utoipa::OpenApi)]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    let mut api = ApiDoc::openapi();
    api.merge(get::openapi());
    api.merge(delete::openapi());
    api
}

#[derive(Error, Debug)]
pub enum UserDataError {
    #[error("Account not found")]
    NotFound,
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for UserDataError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let msg = self.to_string();

        (status, Json(msg)).into_response()
    }
}

#[derive(Serialize, sqlx::FromRow, utoipa::ToSchema)]
struct UserDataResponse {
    email: String,
    id: u64,
}

pub mod delete;
pub mod get;
