use crate::{
    ApiResult, AppState,
    users::{self, Response, Role, auth::validate},
};
use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};

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
            body = Response
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
    _: Role,
) -> ApiResult<impl IntoResponse> {
    let token = validate::extract_session_token(&headers)?;

    let (id, email, role): (u64, String, Role) = sqlx::query_as(
        "SELECT accounts.id, accounts.email, accounts.role FROM sessions JOIN accounts ON accounts.id = sessions.account_id WHERE sessions.token = ? LIMIT 1",
    )
    .bind(token)
    .fetch_one(&pool)
    .await
    .map_err(users::Error::Database)?;

    Ok(Json(Response {
        id,
        email,
        role,

        role_rank: u8::from(role),
    }))
}
