use crate::{api::Context, error::Error, session::SessionAuth};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing, Json, Router,
};
use colette_core::feeds::{CreateFeedDto, FeedsService};
use std::sync::Arc;

pub fn router() -> Router<Context> {
    Router::new()
        .route("/feeds", routing::get(list).post(create))
        .route("/feeds/:id", routing::get(get))
}

#[axum::debug_handler]
async fn list(
    State(service): State<Arc<FeedsService>>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    let feeds = service.list(session).await?;

    Ok(Json(feeds))
}

#[axum::debug_handler]
async fn get(
    State(service): State<Arc<FeedsService>>,
    Path(id): Path<String>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    let feed = service.get(id, session).await?;

    Ok(Json(feed))
}

#[axum::debug_handler]
async fn create(
    State(service): State<Arc<FeedsService>>,
    SessionAuth(session): SessionAuth,
    Json(body): Json<CreateFeedDto>,
) -> Result<impl IntoResponse, Error> {
    let feed = service.create(body, session).await?;

    Ok(Json(feed))
}
