use axum::{routing, Router};
pub use model::Feed;

use crate::common::Context;

mod handler;
mod model;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        handler::list_feeds,
        handler::get_feed,
        handler::create_feed,
        handler::delete_feed
    ),
    components(schemas(Feed, model::CreateFeed))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/feeds",
            Router::new()
                .route(
                    "/",
                    routing::get(handler::list_feeds).post(handler::create_feed),
                )
                .route(
                    "/:id",
                    routing::get(handler::get_feed).delete(handler::delete_feed),
                ),
        )
    }
}
