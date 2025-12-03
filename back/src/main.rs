mod auth;
mod donations;
mod supporters;
use axum::{
    Router,
    http::{self, HeaderValue, Method, header, request::Parts},
    response::{IntoResponse, Response},
    routing,
};
mod health;
mod me;
use sqlx::MySqlPool;
use std::{env, net::SocketAddr};
use thiserror::Error;
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfig};
use tower_http::cors::{AllowOrigin, CorsLayer};
use utoipa_swagger_ui::SwaggerUi;

#[derive(utoipa::OpenApi)]
struct ApiDoc;
fn openapi() -> utoipa::openapi::OpenApi {
    use crate::auth;
    use utoipa::OpenApi;
    let mut api = ApiDoc::openapi();
    api.merge(auth::signup::openapi());
    api.merge(auth::signin::openapi());
    api.merge(auth::signout::openapi());
    api.merge(auth::validate::openapi());
    api.merge(me::openapi());
    api.merge(health::openapi());
    api.merge(donations::openapi());
    api.merge(supporters::openapi());
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
        .route("/health", routing::get(health::health))
        .route("/auth/signup", routing::post(auth::signup))
        .route("/auth/signin", routing::post(auth::signin))
        .route("/auth/signout", routing::post(auth::signout))
        .route("/auth/validate", routing::get(auth::validate))
        .route("/me", routing::get(me::me))
        .route("/donations", routing::get(donations::donations))
        .route("/donations", routing::post(donations::add_donation))
        .route("/supporters", routing::get(supporters::supporters))
        .with_state(pool)
        .layer(GovernorLayer::new(GovernorConfig::default()))
        .layer(
            CorsLayer::new()
                .allow_origin(if let Ok(v) = env::var("CORS_ALLOWED_ORIGINS") {
                    v.split_whitespace()
                        .map(|v| HeaderValue::from_str(v).expect("Invalid CORS_ALLOWED_ORIGINS"))
                        .collect::<Vec<_>>()
                        .into()
                } else {
                    #[cfg(not(debug_assertions))]
                    panic!("CORS_ALLOWED_ORIGINS must be set");
                    #[allow(unreachable_code)]
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

type ApiResult<T> = Result<T, ApiError>;

#[derive(Error, Debug)]
enum ApiError {
    #[error("could not validate session: {0}")]
    Validation(#[from] auth::validate::ValidationError),
    #[error("could not sign in: {0}")]
    Signin(#[from] auth::signin::SigninError),
    #[error("could not sign up: {0}")]
    Signup(#[from] auth::signup::SignupError),
    #[error("could not get donations: {0}")]
    Donation(#[from] donations::DonationError),
    #[error("could not get supporters: {0}")]
    Supporter(#[from] supporters::SupporterError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Validation(e) => e.into_response(),
            ApiError::Signin(e) => e.into_response(),
            ApiError::Signup(e) => e.into_response(),
            ApiError::Donation(e) => e.into_response(),
            ApiError::Supporter(e) => e.into_response(),
        }
    }
}
