use crate::{
    ApiError, ApiResult, AppState, ErrorResponse,
    users::{UserRole, auth::validate},
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::WithRejection as Rejectable;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(utoipa::OpenApi)]
#[openapi(paths(invite))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
struct InviteResponse {
    code: String,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct InviteRequest {
    role: UserRole,
}

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
#[strum(prefix = "INVITE_")]
pub enum Error {
    #[error("Invite already exists")]
    Conflict,
    #[error("Could not query database")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Conflict => StatusCode::CONFLICT,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let error = self.as_ref().to_string();
        let message = self.to_string();

        (status, Json(ErrorResponse { error, message })).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/users/auth/invite",
    responses(
        (
            status = StatusCode::CREATED,
            description = "Successfully generated invite",
            body = InviteResponse,
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ),
)]
pub async fn invite(
    State(AppState { pool }): State<AppState>,
    requester: UserRole,
    Rejectable(Json(InviteRequest { role }), _): Rejectable<Json<InviteRequest>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    if requester < UserRole::SuperAdmin || role >= UserRole::SuperAdmin {
        Err(validate::Error::InsufficientPermissions)?
    }

    let code: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();

    match sqlx::query(
        "INSERT INTO invites (role, code, expires_at) VALUES (?, ?, NOW() + INTERVAL 1 WEEK)",
    )
    .bind(role)
    .bind(&code)
    .execute(&pool)
    .await
    {
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => Err(Error::Conflict)?,
        Err(e) => Err(Error::DatabaseError(e))?,
        Ok(_) => Ok((StatusCode::CREATED, Json(InviteResponse { code }))),
    }
}
