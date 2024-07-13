use axum::{routing, Router};
pub use model::Entry;

use crate::common::Context;

mod handler;
mod model;

#[derive(utoipa::OpenApi)]
#[openapi(paths(handler::list_entries), components(schemas(Entry)))]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/entries",
            Router::new().route("/", routing::get(handler::list_entries)),
        )
    }
}
