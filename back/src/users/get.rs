use crate::{
    ApiError, ApiResult, AppState,
    users::{self, UserDataResponse, UserRole, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::WithRejection as Rejectable;

#[derive(utoipa::OpenApi)]
#[openapi(paths(users, user))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<UserDataResponse>,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = "Insufficient permissions",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn users(
    State(AppState { pool }): State<AppState>,
    role: UserRole,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Admin {
        Err(validate::Error::InsufficientPermissions)?
    }

    Ok(Json(
        sqlx::query_as::<_, (u64, String, UserRole)>("SELECT id, email, role FROM accounts")
            .fetch_all(&pool)
            .await
            .map_err(users::Error::Database)?
            .into_iter()
            .map(|(id, email, role)| UserDataResponse {
                id,
                email,
                role,
                role_rank: u8::from(role),
            })
            .collect::<Vec<_>>(),
    ))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    responses(
        (
            status = StatusCode::OK,
            body = UserDataResponse,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "User not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (
            status = StatusCode::FORBIDDEN,
            description = "Insufficient permissions",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn user(
    State(AppState { pool }): State<AppState>,
    Rejectable(Path(id), _): Rejectable<Path<u64>, ApiError>,
    role: UserRole,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Admin {
        Err(validate::Error::InsufficientPermissions)?
    }

    let user: (u64, String, UserRole) =
        sqlx::query_as("SELECT id, email, role FROM accounts WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(users::Error::Database)?
            .ok_or(users::Error::NotFound)?;

    let (id, email, role) = user;

    let user = UserDataResponse {
        id,
        email,
        role,
        role_rank: u8::from(role),
    };

    Ok((StatusCode::OK, Json(user)))
}
