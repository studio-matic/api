use axum::http::StatusCode;

#[derive(utoipa::OpenApi)]
#[openapi(paths(health))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    head,
    path = "/health",
    responses(
        (status = StatusCode::OK, description = "Backend is healthy"),
    ),
)]
pub async fn health() -> StatusCode {
    StatusCode::OK
}
