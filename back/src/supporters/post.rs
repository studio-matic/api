use crate::{
    ApiResult, AppState,
    supporters::{self, SupporterRequest},
    users::{UserRole, auth::validate},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde::Serialize;

#[derive(utoipa::OpenApi)]
#[openapi(paths(supporter))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
struct SupporterIdResponse {
    id: u64,
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
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn supporter(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
    Json(SupporterRequest { name, donation_id }): Json<SupporterRequest>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    let id = sqlx::query(
        "INSERT INTO supporters (name, donation_id)
        VALUES (?, ?)",
    )
    .bind(name)
    .bind(donation_id)
    .execute(&pool)
    .await
    .map_err(supporters::Error::Database)?
    .last_insert_id();

    Ok((StatusCode::CREATED, Json(SupporterIdResponse { id })))
}
