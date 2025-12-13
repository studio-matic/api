use crate::{
    ApiResult,
    users::{UserDataError, UserDataResponse, UserRole, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use sqlx::MySqlPool;

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
    State(pool): State<MySqlPool>,
    role: UserRole,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Admin {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let users: Vec<(u64, String, UserRole)> =
        sqlx::query_as("SELECT id, email, role FROM accounts")
            .fetch_all(&pool)
            .await
            .map_err(UserDataError::DatabaseError)?;

    let users = users
        .into_iter()
        .map(|(id, email, role)| UserDataResponse {
            id,
            email,
            role,
            role_rank: u8::from(role),
        })
        .collect::<Vec<_>>();

    Ok((StatusCode::OK, Json(users)))
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
    State(pool): State<MySqlPool>,
    role: UserRole,
    Path(id): Path<u64>,
) -> ApiResult<impl IntoResponse> {
    if role < UserRole::Admin {
        Err(validate::ValidationError::InsufficientPermissions)?;
    }

    let user: (u64, String, UserRole) =
        sqlx::query_as("SELECT id, email, role FROM accounts WHERE id = ? LIMIT 1")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .map_err(UserDataError::DatabaseError)?
            .ok_or(UserDataError::NotFound)?;

    let (id, email, role) = user;

    let user = UserDataResponse {
        id,
        email,
        role,
        role_rank: u8::from(role),
    };

    Ok((StatusCode::OK, Json(user)))
}
