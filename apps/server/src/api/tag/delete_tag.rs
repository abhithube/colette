use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag::{self, TagService};

use crate::api::common::{BaseError, Error, Id, Session, TAGS_TAG};

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteTag",
    description = "Delete a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<TagService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_tag(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted tag")]
    NoContent,

    #[response(status = 404, description = "Tag not found")]
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
