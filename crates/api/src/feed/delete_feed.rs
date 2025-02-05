use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed::{self, FeedService};

use crate::common::{BaseError, Error, FEEDS_TAG, Id, Session};

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteFeed",
    description = "Delete a feed by ID",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<FeedService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_feed(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted feed")]
    NoContent,

    #[response(status = 404, description = "Feed not found")]
    NotFound(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
