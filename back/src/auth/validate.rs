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
        (status = StatusCode::OK, description = "Successful login", body = ValidationResponse),
        (
            status = StatusCode::UNAUTHORIZED, 
            description = "Unsuccessful login: session_token cookie not found | Unsuccessful login: Session token not found | Unsuccessful login: Cookies not found", 
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
        match sqlx::query(
            "SELECT accounts.email
            FROM accounts
            INNER JOIN sessions
            ON accounts.id = sessions.account_id
            WHERE token = ?"
        )
            .bind(session_token)
            .fetch_optional(&pool)
            .await
            .ok().flatten()
        {
            Some(email) => (
                StatusCode::OK,
                Json(ValidationResponse {
                    email: email.get::<String, _>(0),
                    message: "Successful login",
                }),
            )
                .into_response(),
            None => (
                StatusCode::UNAUTHORIZED,
                Json("Unsuccessful login: Session token not found"),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json("Unsuccessful login: Cookies not found"),
        )
            .into_response()
    }
}
