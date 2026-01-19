use crate::{
    ApiError, ApiResult, AppState,
    users::{self, Response, Role, auth::validate, email::EmailAddress},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use axum_extra::extract::WithRejection as Rejectable;
use serde::Deserialize;

#[derive(utoipa::OpenApi)]
#[openapi(paths(me))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Deserialize, utoipa::ToSchema)]
#[schema(as = users::me::Request)]
pub struct Request {
    email: Option<EmailAddress>,
    password: Option<String>,
}

#[utoipa::path(
    patch,
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
    Rejectable(Json(Request { email, password }), _): Rejectable<Json<Request>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    let token = validate::extract_session_token(&headers)?;

    let mut transaction = pool.begin().await.map_err(users::Error::Database)?;
    let id: u64 = sqlx::query_scalar(
        "SELECT accounts.id FROM sessions JOIN accounts ON accounts.id = sessions.account_id WHERE sessions.token = ? LIMIT 1",
    )
    .bind(token)
    .fetch_one(&pool)
    .await
    .map_err(users::Error::Database)?;

    if let Some(email) = email {
        let _ = sqlx::query("UPDATE accounts SET email = ? WHERE id = ? LIMIT 1")
            .bind(email)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(users::Error::Database)?;
    }
    if let Some(password) = password {
        let password = Argon2::default()
            .hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map_err(users::Error::PasswordHash)?
            .to_string();

        let _ = sqlx::query("UPDATE accounts SET password = ? WHERE id = ? LIMIT 1")
            .bind(password)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(users::Error::Database)?;
    }

    transaction.commit().await.map_err(users::Error::Database)?;
    Ok(())
}
