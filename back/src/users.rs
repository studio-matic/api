use crate::ErrorResponse;
use axum::{
    Json,
    http::StatusCode,
    response::{self, IntoResponse},
};
use serde::{Deserialize, Serialize};
use sqlx::Type;

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

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[strum(prefix = "USERS_")]
pub enum Error {
    #[error("Account not found")]
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
#[schema(as = users::Response)]
struct Response {
    id: u64,
    email: String,
    role: Role,
    role_rank: u8,
}

#[derive(
    Clone,
    Copy,
    Serialize,
    Deserialize,
    Type,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    utoipa::ToSchema,
    Debug,
)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum Role {
    None,
    Editor,
    Admin,
    SuperAdmin,
}

// HACK: to implement hierarchical `>` and `<` for `WHERE` clauses
impl From<Role> for u8 {
    fn from(role: Role) -> Self {
        match role {
            Role::None => 1,
            Role::Editor => 2,
            Role::Admin => 3,
            Role::SuperAdmin => 4,
        }
    }
}

pub mod delete;
pub mod get;
