use crate::{
    ApiError, ApiResult, AppState, donations,
    users::{UserRole, auth::validate},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::WithRejection as Rejectable;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    delete,
    path = "/donations/{id}",
    responses(
        (
            status = StatusCode::NO_CONTENT,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Donation not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn donation(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
    Rejectable(Path(id), _): Rejectable<Path<u64>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    Ok(sqlx::query("DELETE FROM donations WHERE id = ? LIMIT 1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(donations::Error::Database)?
        .rows_affected()
        .ne(&0)
        .then_some(StatusCode::NO_CONTENT)
        .ok_or(donations::Error::NotFound)?)
}
