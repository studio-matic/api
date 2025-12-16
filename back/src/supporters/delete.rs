use crate::{
    ApiError, ApiResult, AppState, supporters,
    users::{UserRole, auth::validate},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::WithRejection as Rejectable;

#[derive(utoipa::OpenApi)]
#[openapi(paths(supporter))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    delete,
    path = "/supporters/{id}",
    responses(
        (
            status = StatusCode::NO_CONTENT,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Supporter not found",
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
    Rejectable(Path(id), _): Rejectable<Path<u64>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    Ok(sqlx::query("DELETE FROM supporters WHERE id = ? LIMIT 1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(supporters::Error::Database)?
        .rows_affected()
        .ne(&0)
        .then_some(StatusCode::NO_CONTENT)
        .ok_or(supporters::Error::NotFound)?)
}
