use crate::{
    ApiError, ApiResult, AppState,
    supporters::{self, SupporterRequest},
    users::{UserRole, auth::validate},
};
use axum::{
    Json,
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
    put,
    path = "/supporters/{id}",
    responses(
        (
            status = StatusCode::OK,
            description = "Successfully added supporter",
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
    Rejectable(Json(SupporterRequest { name, donation_id }), _): Rejectable<
        Json<SupporterRequest>,
        ApiError,
    >,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    Ok(
        sqlx::query("UPDATE supporters  SET  name = ?, donation_id = ? WHERE id = ?")
            .bind(name)
            .bind(donation_id)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(supporters::Error::Database)?
            .rows_affected()
            .ne(&0)
            .then_some(StatusCode::OK)
            .ok_or(supporters::Error::NotFound)?,
    )
}
