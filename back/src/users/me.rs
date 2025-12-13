use crate::{
    ApiResult, AppState,
    users::{UserDataError, UserDataResponse, UserRole, auth::validate},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(me))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    get,
    path = "/users/me",
    responses(
        (
            status = StatusCode::OK,
            body = UserDataResponse
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn me(
    State(AppState { pool }): State<AppState>,
    headers: HeaderMap,
    _: UserRole,
) -> ApiResult<impl IntoResponse> {
    let token = validate::extract_session_token(&headers)?;

    let user: (u64, String, UserRole) = sqlx::query_as(
        "SELECT accounts.id, accounts.email, accounts.role
            FROM sessions
            JOIN accounts ON accounts.id = sessions.account_id
            WHERE sessions.token = ?
            LIMIT 1",
    )
    .bind(token)
    .fetch_one(&pool)
    .await
    .map_err(UserDataError::DatabaseError)?;

    let (id, email, role) = user;

    let user = UserDataResponse {
        id,
        email,
        role,

        role_rank: u8::from(role),
    };

    Ok((StatusCode::OK, Json(user)))
}
