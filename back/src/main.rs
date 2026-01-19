mod donations;
mod supporters;
use axum::{
    Json, Router,
    extract::rejection,
    http::{self, HeaderValue, Method, header, request::Parts},
    response::{IntoResponse, Response},
    routing,
};
mod health;
mod users;
use serde::Serialize;
use sqlx::MySqlPool;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfig};
use tower_http::cors::{AllowOrigin, CorsLayer};
use utoipa::{
    PartialSchema, ToSchema,
    openapi::{Components, RefOr, Schema},
};
use utoipa_swagger_ui::SwaggerUi;

#[derive(utoipa::OpenApi)]
struct ApiDoc;
fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    let mut api = ApiDoc::openapi();
    let _ = api
        .components
        .insert(Components::builder().schema_from::<ApiError>().build());
    api.merge(users::auth::invite::openapi());
    api.merge(users::auth::signup::openapi());
    api.merge(users::auth::signin::openapi());
    api.merge(users::auth::signout::openapi());
    api.merge(users::auth::validate::openapi());
    api.merge(users::me::openapi());
    api.merge(users::openapi());
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

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Unable to perform mysql database migrations");

    tokio::spawn(users::auth::cleanup_expired_sessions(pool.clone()));

    let app = Router::new()
        .merge(SwaggerUi::new("/").url("/api-docs/openapi.json", openapi()))
        .route("/health", routing::head(health::health))
        .route("/users", routing::get(users::get::users))
        .route("/users/{id}", routing::get(users::get::user))
        .route("/users/{id}", routing::delete(users::delete::user))
        .route("/users/auth/invite", routing::post(users::auth::invite))
        .route("/users/auth/signup", routing::post(users::auth::signup))
        .route("/users/auth/signin", routing::post(users::auth::signin))
        .route("/users/auth/signout", routing::delete(users::auth::signout))
        .route("/users/auth/validate", routing::get(users::auth::validate))
        .route("/users/me", routing::get(users::me::get::me))
        .route("/users/me", routing::patch(users::me::patch::me))
        .route("/donations", routing::get(donations::get::donations))
        .route("/donations/{id}", routing::get(donations::get::donation))
        .route("/donations", routing::post(donations::post::donation))
        .route("/donations/{id}", routing::put(donations::put::donation))
        .route(
            "/donations/{id}",
            routing::delete(donations::delete::donation),
        )
        .route("/supporters", routing::get(supporters::get::supporters))
        .route("/supporters/{id}", routing::get(supporters::get::supporter))
        .route("/supporters", routing::post(supporters::post::supporter))
        .route("/supporters/{id}", routing::put(supporters::put::supporter))
        .route(
            "/supporters/{id}",
            routing::delete(supporters::delete::supporter),
        )
        .with_state(AppState { pool })
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
                .allow_methods([
                    Method::GET,
                    Method::HEAD,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                ])
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

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    error: String,
    message: String,
}

type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error, strum::AsRefStr, strum::VariantNames)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiError {
    #[error("Could not validate session: {0}")]
    Validation(#[from] users::auth::validate::Error),
    #[error("Could not retreive invite: {0}")]
    Invite(#[from] users::auth::invite::Error),
    #[error("Could not sign in: {0}")]
    Signin(#[from] users::auth::signin::Error),
    #[error("Could not sign up: {0}")]
    Signup(#[from] users::auth::signup::Error),
    #[error("Could not retreive user data: {0}")]
    UserData(#[from] users::Error),
    #[error("Could not get donations: {0}")]
    Donation(#[from] donations::Error),
    #[error("Could not get supporters: {0}")]
    Supporter(#[from] supporters::Error),
    #[error("Could not deserialize json: {0}")]
    Json(#[from] rejection::JsonRejection),
    #[error("Could not match path: {0}")]
    Path(#[from] rejection::PathRejection),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Validation(e) => e.into_response(),
            ApiError::Invite(e) => e.into_response(),
            ApiError::Signin(e) => e.into_response(),
            ApiError::Signup(e) => e.into_response(),
            ApiError::UserData(e) => e.into_response(),
            ApiError::Donation(e) => e.into_response(),
            ApiError::Supporter(e) => e.into_response(),
            ApiError::Json(ref e) => {
                let error = self.as_ref().to_string();
                let message = self.to_string();
                (e.status(), Json(ErrorResponse { error, message })).into_response()
            }
            ApiError::Path(ref e) => {
                let error = self.as_ref().to_string();
                let message = self.to_string();
                (e.status(), Json(ErrorResponse { error, message })).into_response()
            }
        }
    }
}

impl PartialSchema for ApiError {
    fn schema() -> RefOr<Schema> {
        let variants = [
            <users::auth::validate::Error as strum::VariantNames>::VARIANTS,
            <users::auth::invite::Error as strum::VariantNames>::VARIANTS,
            <users::auth::signin::Error as strum::VariantNames>::VARIANTS,
            <users::auth::signup::Error as strum::VariantNames>::VARIANTS,
            <users::Error as strum::VariantNames>::VARIANTS,
            <donations::Error as strum::VariantNames>::VARIANTS,
            <supporters::Error as strum::VariantNames>::VARIANTS,
        ]
        .into_iter()
        .flat_map(IntoIterator::into_iter)
        .chain(["JSON_REJECTION", "PATH_REJECTION"].iter())
        .map(ToString::to_string)
        .collect::<Vec<String>>();

        RefOr::T(Schema::from(
            utoipa::openapi::Object::builder()
                .schema_type(utoipa::openapi::Type::String)
                .enum_values(Some(variants))
                .build(),
        ))
    }
}

impl ToSchema for ApiError {
    fn name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("ApiError")
    }
    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.extend([]);
    }
}

#[derive(Clone)]
pub struct AppState {
    pool: MySqlPool,
}
