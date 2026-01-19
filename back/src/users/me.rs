#[derive(utoipa::OpenApi)]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    use utoipa::OpenApi;
    let mut api = ApiDoc::openapi();
    api.merge(get::openapi());
    api.merge(patch::openapi());
    api
}

pub mod get;
pub mod patch;
