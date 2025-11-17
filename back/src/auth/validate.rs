use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::{MySqlPool, Row};

#[derive(utoipa::OpenApi)]
#[openapi(paths(validate))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
struct ValidationResponse {
    email: String,
    message: &'static str,
}

#[utoipa::path(
    get,
    path = "/validate",
    responses(
        (status = StatusCode::OK, description = "successful login", body = ValidationResponse),
        (
            status = StatusCode::UNAUTHORIZED, 
            description = "Unsuccessful login: session_token cookie not found | Unsuccessful login: session token not found | Unsuccessful login: cookies not found", 
        ),
    ),
)]
pub async fn validate(State(pool): State<MySqlPool>, headers: HeaderMap) -> impl IntoResponse {
    if let Some(cookie_header) = headers.get(header::COOKIE) {
        let cookies = cookie_header.to_str().unwrap_or_default();
        let session_token = if let Some(v) = cookies
            .split(';')
            .find_map(|s| s.trim().strip_prefix("session_token="))
        {
            v
        } else {
            return (
                StatusCode::UNAUTHORIZED,
                Json("Unsuccessful login: session_token cookie not found"),
            )
                .into_response();
        };
        match sqlx::query("SELECT (email) FROM sessions WHERE token = ?")
            .bind(session_token)
            .fetch_optional(&pool)
            .await
            .ok().flatten()
        {
            Some(email) => (
                StatusCode::OK,
                Json(ValidationResponse {
                    email: email.get::<String, _>(0),
                    message: "successful login",
                }),
            )
                .into_response(),
            None => (
                StatusCode::UNAUTHORIZED,
                Json("Unsuccessful login: session token not found"),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json("Unsuccessful login: cookies not found"),
        )
            .into_response()
    }
}
