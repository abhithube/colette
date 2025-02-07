use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed_entry::{self, FeedEntryService};

use super::FeedEntry;
use crate::api::common::{BaseError, Error, FEED_ENTRIES_TAG, Id, Session};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFeedEntry",
    description = "Get a feed entry by ID",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<FeedEntryService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_feed_entry(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            feed_entry::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Feed entry by ID")]
    Ok(FeedEntry),

    #[response(status = 404, description = "Feed entry not found")]
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
