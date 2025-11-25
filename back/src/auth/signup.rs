use super::{SESSION_TOKEN_MAX_AGE, SignRequest, generate_session_token};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};
use sqlx::MySqlPool;

#[derive(utoipa::OpenApi)]
#[openapi(paths(signup))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    post,
    path = "/signup",
    responses(
        (
            status = StatusCode::CREATED, description = "Successful signup",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            description = "Internal server error | Unsuccessful signup, but could not save session token",
        ),
        (
            status = StatusCode::CONFLICT,
            description = "Unsuccessful signup: Account already exists",
        ),
    ),
)]
pub async fn signup(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let token = generate_session_token();

    let hashed_password = Argon2::default()
        .hash_password(req.password.as_bytes(), &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();

    let account_result = sqlx::query("INSERT INTO accounts (email, password) VALUES (?, ?)")
        .bind(&req.email)
        .bind(&hashed_password)
        .execute(&pool)
        .await;

    match account_result {
        Ok(_) => {
            if let Err(e) = sqlx::query("INSERT INTO sessions (token, email) VALUES (?, ?)")
                .bind(&token)
                .bind(&req.email)
                .execute(&pool)
                .await
            {
                eprintln!("{e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Unsuccessful signup, but could not save session token"),
                )
                    .into_response();
            }
            (
                StatusCode::CREATED,
                AppendHeaders([(
                    header::SET_COOKIE,
                    #[cfg(debug_assertions)]
                    format!(
                        "session_token={token}; Max-Age={}; HttpOnly",
                        SESSION_TOKEN_MAX_AGE.as_secs()
                    ),
                    #[cfg(not(debug_assertions))]
                    format!(
                        "session_token={token}; Max-Age={}; HttpOnly; Secure; SameSite=None",
                        SESSION_TOKEN_MAX_AGE.as_secs()
                    ),
                )]),
                Json("Successful signup"),
            )
                .into_response()
        }
        Err(sqlx::Error::Database(e)) if e.is_unique_violation() => {
            (StatusCode::CONFLICT, Json("Unsuccessful signup: Account already exists")).into_response()
        }
        Err(e) => {
            eprintln!("{e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Internal server error"),
            )
                .into_response()
        }
    }
}
