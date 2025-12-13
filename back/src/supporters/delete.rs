use crate::{
    ApiResult, AppState,
    supporters::SupporterError,
    users::{UserRole, auth::validate},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

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
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let res = sqlx::query("DELETE FROM supporters WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(SupporterError::DatabaseError)?;

    if res.rows_affected() == 0 {
        Err(SupporterError::NotFound.into())
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
