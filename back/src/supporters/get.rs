use crate::{
    ApiResult, AppState,
    supporters::{SupporterError, SupporterResponse},
    users::{UserRole, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(supporters, supporter))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    get,
    path = "/supporters",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<SupporterResponse>,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn supporters(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let supporters: Vec<(u64, String, u64)> =
        sqlx::query_as("SELECT id, name, donation_id FROM supporters")
            .fetch_all(&pool)
            .await
            .map_err(SupporterError::DatabaseError)?;

    let supporters = supporters
        .into_iter()
        .map(|(a, b, c)| {
            Ok(SupporterResponse {
                id: a,
                name: b,
                donation_id: c,
            })
        })
        .collect::<ApiResult<Vec<_>>>()?;

    Ok((StatusCode::OK, Json(supporters)))
}

#[utoipa::path(
    get,
    path = "/supporters/{id}",
    responses(
        (
            status = StatusCode::OK,
            body = SupporterResponse,
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
    ),
)]
pub async fn supporter(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Editor {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let supporter: (u64, String, u64) = sqlx::query_as(
        "SELECT id, name, donation_id FROM supporters WHERE supporters.id = ? LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(SupporterError::DatabaseError)?
    .ok_or(SupporterError::NotFound)?;

    let (id, name, donation_id) = supporter;

    let supporter = SupporterResponse {
        id,
        name,
        donation_id,
    };

    Ok((StatusCode::OK, Json(supporter)))
}
