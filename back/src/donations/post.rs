use crate::{
    ApiError, ApiResult, AppState,
    donations::{self, Request},
    users::{Role, auth::validate},
};
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::WithRejection as Rejectable;
use serde::Serialize;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}

#[derive(Serialize, utoipa::ToSchema)]
#[schema(as = donations::IdResponse)]
struct IdResponse {
    id: u64,
}

#[utoipa::path(
    post,
    path = "/donations",
    responses(
        (
            status = StatusCode::CREATED,
            body = IdResponse,
            description = "Successfully added donation",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    )
)]
pub async fn donation(
    State(AppState { pool }): State<AppState>,
    role: Role,
    Rejectable(
        Json(Request {
            coins,
            income_eur,
            co_op,
        }),
        _,
    ): Rejectable<Json<Request>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    if role < Role::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    let id = sqlx::query(
        "INSERT INTO donations (coins, income_eur, co_op)
        VALUES (?, ?, ?)",
    )
    .bind(coins)
    .bind(income_eur)
    .bind(co_op)
    .execute(&pool)
    .await
    .map_err(donations::Error::Database)?
    .last_insert_id();

    Ok((StatusCode::CREATED, Json(IdResponse { id })))
}
