use crate::{
    ApiResult,
    supporters::{SupporterError, SupporterRequest},
    users::{UserRole, auth::validate},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::MySqlPool;

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
    State(pool): State<MySqlPool>,
    headers: HeaderMap,
    Json(req): Json<SupporterRequest>,
) -> ApiResult<impl IntoResponse> {
    let _ = validate::validate_role(&pool, headers, UserRole::Editor).await?;

    let id = sqlx::query(
        "INSERT INTO supporters (name, donation_id)
        VALUES (?, ?)",
    )
    .bind(req.name)
    .bind(req.donation_id)
    .execute(&pool)
    .await
    .map_err(SupporterError::DatabaseError)?
    .last_insert_id();

    Ok((StatusCode::CREATED, Json(SupporterIdResponse { id })))
}
