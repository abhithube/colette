use axum::{routing, Router};
use utoipa::OpenApi;

use crate::api::Context;

mod handler;
mod model;

#[derive(OpenApi)]
#[openapi(
    paths(handler::register, handler::login),
    components(schemas(model::Register, model::Login, model::User))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/auth",
            Router::new()
                .route("/register", routing::post(handler::register))
                .route("/login", routing::post(handler::login)),
        )
    }
}
