use axum::{routing, Router};
pub use model::FeedDto;
use utoipa::OpenApi;

use crate::api::Context;

mod handler;
mod model;

#[derive(OpenApi)]
#[openapi(
    paths(
        handler::list_feeds,
        handler::get_feed,
        handler::create_feed,
        handler::delete_feed
    ),
    components(schemas(model::FeedDto, model::CreateFeedDto))
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
