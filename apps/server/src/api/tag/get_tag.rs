use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag::{self, TagService};

use super::Tag;
use crate::api::common::{BaseError, Error, Id, Session, TAGS_TAG};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getTag",
    description = "Get a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<TagService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_tag(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Tag by ID")]
    Ok(Tag),

    #[response(status = 404, description = "Tag not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
