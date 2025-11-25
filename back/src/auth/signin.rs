use super::{SESSION_TOKEN_MAX_AGE, SignRequest, generate_session_token};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{AppendHeaders, IntoResponse},
};
use sqlx::{MySqlPool, Row};

#[derive(utoipa::OpenApi)]
#[openapi(paths(signin))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[utoipa::path(
    post,
    path = "/signin",
    responses(
        (
            status = StatusCode::NOT_FOUND,
            description = "Unsuccessful signin: Account not found",
        ),
        (
            status = StatusCode::INTERNAL_SERVER_ERROR,
            description = "Successful signin, but could not save session token",
        ),
        (
            status = StatusCode::OK,
            description = "Successful signin"
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Unsuccessful signin: Password incorrect",
        ),
    ),
)]
pub async fn signin(
    State(pool): State<MySqlPool>,
    Json(req): Json<SignRequest>,
) -> impl IntoResponse {
    let hashed_password = if let Some(v) =
        sqlx::query("SELECT (password) FROM accounts WHERE email = ?")
            .bind(&req.email)
            .fetch_optional(&pool)
            .await
            .ok().flatten()
    {
        v
    } else {
        return (
            StatusCode::NOT_FOUND,
            Json("Unsuccessful signin: Account not found"),
        )
            .into_response();
    }
    .get::<String, _>(0);

    let token = generate_session_token();

    if Argon2::default()
        .verify_password(
            req.password.as_bytes(),
            &PasswordHash::new(&hashed_password).unwrap(),
        )
        .is_ok()
    {
        if let Err(e) = sqlx::query("INSERT INTO sessions (token, email) VALUES (?, ?)")
            .bind(&token)
            .bind(&req.email)
            .execute(&pool)
            .await
        {
            eprintln!("{e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json("Successful signin, but could not save session token"),
            )
                .into_response()
        } else {
            (
                StatusCode::OK,
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
                Json("Successful signin"),
            )
                .into_response()
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json("Unsuccessful signin: Password incorrect"),
        )
            .into_response()
    }
}
