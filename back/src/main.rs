mod auth;
use axum::{
    Json, Router,
    http::{self, HeaderValue, Method, StatusCode, header, request::Parts},
    routing,
};
use serde::Serialize;
use sqlx::MySqlPool;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfig};
use tower_http::cors::{AllowOrigin, CorsLayer};
use utoipa_swagger_ui::SwaggerUi;

#[derive(utoipa::OpenApi)]
#[openapi(paths(health))]
struct ApiDoc;
fn openapi() -> utoipa::openapi::OpenApi {
    use crate::auth;
    use utoipa::OpenApi;
    let mut api = ApiDoc::openapi();
    api.merge(auth::signup::openapi());
    api.merge(auth::signin::openapi());
    api.merge(auth::validate::openapi());
    api
}

#[tokio::main]
async fn main() {
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("Unable to connect to mysql database");

    tokio::spawn(auth::cleanup_expired_sessions(pool.clone()));

    let app = Router::new()
        .merge(SwaggerUi::new("/").url("/api-docs/openapi.json", openapi()))
        .route("/health", routing::get(health))
        .route("/auth/signup", routing::post(auth::signup))
        .route("/auth/signin", routing::post(auth::signin))
        .route("/auth/validate", routing::get(auth::validate))
        .with_state(pool)
        .layer(GovernorLayer::new(GovernorConfig::default()))
        .layer(
            CorsLayer::new()
                .allow_origin(if let Ok(e) = env::var("CORS_ALLOWED_ORIGIN") {
                    HeaderValue::from_str(&e)
                        .expect("Invalid CORS_ALLOWED_ORIGIN")
                        .into()
                } else {
                    #[cfg(not(debug_assertions))]
                    panic!("CORS_ALLOWED_ORIGIN must be set");
                    #[cfg(debug_assertions)]
                    AllowOrigin::predicate(move |_: &http::HeaderValue, _: &Parts| true)
                })
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::ACCEPT,
                    header::AUTHORIZATION,
                    header::ORIGIN,
                    header::USER_AGENT,
                ])
                .allow_credentials(true),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    let port = env::var("PORT").expect("PORT must be set");
    let listener = TcpListener::bind(format!("[::]:{port}"))
        .await
        .unwrap_or_else(|_| panic!("Unable to bind http://[::]:{port} and 0.0.0.0:{port}"));
    println!("Listening on http://[::]:{port} and http://0.0.0.0:{port} ...");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, utoipa::ToSchema)]
struct Health {
    status: &'static str,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = StatusCode::OK, description = "Backend is healthy", body = Health),
    ),
)]
async fn health() -> (StatusCode, Json<Health>) {
    (StatusCode::OK, Json(Health { status: "ok" }))
}
