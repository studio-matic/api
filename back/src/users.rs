use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::Type;
use thiserror::Error;

pub mod email;

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

#[derive(Serialize, utoipa::ToSchema)]
struct UserDataResponse {
    id: u64,
    email: String,
    role: UserRole,
    role_rank: u8,
}

#[derive(Clone, Copy, Serialize, Type, PartialEq, Eq, PartialOrd, Ord, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    None,
    Editor,
    Admin,
    SuperAdmin,
}

// HACK: to implement hierarchical `>` and `<` for `WHERE` clauses
impl From<UserRole> for u8 {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::None => 1,
            UserRole::Editor => 2,
            UserRole::Admin => 3,
            UserRole::SuperAdmin => 4,
        }
    }
}

pub mod delete;
pub mod get;
