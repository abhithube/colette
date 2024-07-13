use axum::{routing, Router};
pub use model::Collection;

use crate::common::Context;

mod handler;
mod model;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        handler::list_collections,
        handler::get_collection,
        handler::create_collection,
        handler::delete_collection
    ),
    components(schemas(Collection, model::CreateCollection))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/collections",
            Router::new()
                .route(
                    "/",
                    routing::get(handler::list_collections).post(handler::create_collection),
                )
                .route(
                    "/:id",
                    routing::get(handler::get_collection).delete(handler::delete_collection),
                ),
        )
    }
}
