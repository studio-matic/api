use crate::{
    ApiError, ApiResult, AppState,
    donations::{self, Response},
    users::{Role, auth::validate},
};
use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use axum_extra::extract::WithRejection as Rejectable;
use time::OffsetDateTime;

#[derive(utoipa::OpenApi)]
#[openapi(paths(donations, donation))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    ApiDoc::openapi()
}
#[utoipa::path(
    get,
    path = "/donations",
    responses(
        (
            status = StatusCode::OK,
            body = Vec<Response>,
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn donations(
    State(AppState { pool }): State<AppState>,
    role: Role,
) -> ApiResult<impl IntoResponse> {
    if role < Role::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    Ok(Json(
        sqlx::query_as::<_, (u64, u64, OffsetDateTime, f64, String)>(
            "SELECT id, coins, donated_at, income_eur, co_op FROM donations",
        )
        .fetch_all(&pool)
        .await
        .map_err(donations::Error::Database)?
        .into_iter()
        .map(|(a, b, c, d, e)| {
            Ok(Response {
                id: a,
                coins: b,
                donated_at: c
                    .to_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .map_err(donations::Error::TimeFormat)?,
                income_eur: d,
                co_op: e,
            })
        })
        .collect::<ApiResult<Vec<_>>>()?,
    ))
}

#[utoipa::path(
    get,
    path = "/donations/{id}",
    responses(
        (
            status = StatusCode::OK,
            body = Response,
        ),
        (
            status = StatusCode::NOT_FOUND,
            description = "Donation not found",
        ),
        (
            status = StatusCode::UNAUTHORIZED,
            description = "Not logged in",
        ),
        (status = StatusCode::INTERNAL_SERVER_ERROR)
    ),
)]
pub async fn donation(
    State(AppState { pool }): State<AppState>,
    role: Role,
    Rejectable(Path(id), _): Rejectable<Path<u64>, ApiError>,
) -> ApiResult<impl IntoResponse> {
    if role < Role::Editor {
        Err(validate::Error::InsufficientPermissions)?
    }

    let (id, coins, donated_at, income_eur, co_op): (u64, u64, OffsetDateTime, f64, String) =
        sqlx::query_as(
            "SELECT id, coins, donated_at, income_eur, co_op FROM donations WHERE id = ? LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(donations::Error::Database)?
        .ok_or(donations::Error::NotFound)?;

    Ok(Json(Response {
        id,
        coins,
        donated_at: donated_at
            .to_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(donations::Error::TimeFormat)?,
        income_eur,
        co_op,
    }))
}
