use axum::{routing, Router};
pub use model::Bookmark;

use crate::common::Context;

mod handler;
mod model;

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(handler::list_bookmarks, handler::delete_bookmark),
    components(schemas(Bookmark))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/bookmarks",
            Router::new()
                .route("/", routing::get(handler::list_bookmarks))
                .route("/:id", routing::delete(handler::delete_bookmark)),
        )
    }
}
