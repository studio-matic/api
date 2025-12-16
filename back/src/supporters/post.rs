use crate::{
    ApiError, ApiResult, AppState,
    supporters::{self, Request},
    users::{Role, auth::validate},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::WithRejection as Rejectable;
use serde::Serialize;

#[derive(utoipa::OpenApi)]
#[openapi(paths(supporter))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(as = supporters::IdResponse)]
struct IdResponse {
    id: u64,
}

#[utoipa::path(
    post,
    path = "/supporters",
    responses(
        (
            status = StatusCode::CREATED,
            body = IdResponse,
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
    role: Role,
    Rejectable(Json(Request { name, donation_id }), _): Rejectable<Json<Request>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    if role < Role::Editor {
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

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}
