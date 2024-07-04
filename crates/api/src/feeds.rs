use crate::{api::Context, error::Error, session::SessionAuth};
use axum::{extract::State, response::IntoResponse, routing, Json, Router};
use colette_core::feeds::{CreateFeedDto, FeedsService};
use std::sync::Arc;

pub fn router() -> Router<Context> {
    Router::new().route("/feeds", routing::post(create))
}

#[axum::debug_handler]
async fn create(
    State(service): State<Arc<FeedsService>>,
    SessionAuth(session): SessionAuth,
    Json(body): Json<CreateFeedDto>,
) -> Result<impl IntoResponse, Error> {
    let profiles = service.create(body, session).await?;

    Ok(Json(profiles))
}
