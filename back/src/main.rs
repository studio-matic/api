use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderValue, Method},
    routing,
};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfig};
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

#[tokio::main]
async fn main() {
    let pool = MySqlPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL must be set"))
        .await
        .expect("Unable to connect to mysql database");

    let app = Router::new()
        .route("/health", routing::get(health))
        .route("/register", routing::post(register))
        .route("/signup", routing::post(signup))
        .route("/signin", routing::post(signin))
        .with_state(pool)
        .layer(GovernorLayer::new(GovernorConfig::default()))
        .layer(
            CorsLayer::new()
                .allow_origin({
                    let x: AllowOrigin = if let Ok(e) = env::var("FRONT_URL") {
                        HeaderValue::from_str(&e).expect("Invalid FRONT_URL").into()
                    } else {
                        eprintln!("WARNING: FRONT_URL unset, allowing all origins for CORS");
                        Any.into()
                    };
                    x
                })
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any),
        )
        .into_make_service_with_connect_info::<SocketAddr>();

    let port = env::var("PORT").expect("PORT must be set");
    let listener = TcpListener::bind(format!("[::]:{port}"))
        .await
        .unwrap_or_else(|_| panic!("Unable to bind http://[::]:{port} and 0.0.0.0:{port}"));
    println!("Listening on http://[::]:{port} and http://0.0.0.0:{port} ...");
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
}

async fn register(State(pool): State<MySqlPool>, Json(req): Json<RegisterRequest>) -> &'static str {
    let _ = sqlx::query("INSERT INTO registrations (email) VALUES (?)")
        .bind(req.email)
        .execute(&pool)
        .await;

    "ok"
}

#[derive(Deserialize)]
struct SignRequest {
    email: String,
    password: String,
}

async fn signup(State(pool): State<MySqlPool>, Json(req): Json<SignRequest>) -> &'static str {
    match sqlx::query("INSERT INTO accounts (email, password) VALUES (?, ?)")
        .bind(req.email)
        .bind(
            Argon2::default()
                .hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng))
                .unwrap()
                .to_string(),
        )
        .execute(&pool)
        .await
    {
        Ok(_) => "ok",
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => "Account exists already",
        Err(e) => panic!("{e}"),
    }
}

async fn signin(State(pool): State<MySqlPool>, Json(req): Json<SignRequest>) -> &'static str {
    let row = sqlx::query("SELECT (password) FROM accounts WHERE email = ?")
        .bind(req.email)
        .fetch_optional(&pool)
        .await
        .unwrap();
    let hashed_password = match row {
        Some(v) => v,
        None => return "Account not found",
    }
    .get::<String, _>(0);
    match Argon2::default().verify_password(
        req.password.as_bytes(),
        &PasswordHash::new(&hashed_password).unwrap(),
    ) {
        Ok(_) => "Successful login",
        Err(_) => "Unsuccesful login",
    }
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}
