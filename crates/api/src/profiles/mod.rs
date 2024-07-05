use crate::api::Context;
use axum::{routing, Router};
pub use model::Profile;
use utoipa::OpenApi;

mod handler;
mod model;

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::list_profiles,
        handler::get_active_profile,
        handler::create_profile,
        handler::delete_profile
    ),
    components(schemas(model::Profile, model::CreateProfile))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/profiles",
            Router::new()
                .route(
                    "/",
                    routing::get(handler::list_profiles).post(handler::create_profile),
                )
                .route("/@me", routing::get(handler::get_active_profile))
                .route("/:id", routing::delete(handler::delete_profile)),
        )
    }
}
