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

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedEntryUpdate {
    pub has_read: Option<bool>,
}

impl From<FeedEntryUpdate> for feed_entry::FeedEntryUpdate {
    fn from(value: FeedEntryUpdate) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = FeedEntryUpdate,
    responses(UpdateResponse),
    operation_id = "updateFeedEntry",
    description = "Update a feed entry by ID",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<FeedEntryService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<FeedEntryUpdate>,
) -> Result<UpdateResponse, Error> {
    match service
        .update_feed_entry(id, body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            feed_entry::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated feed entry")]
    Ok(FeedEntry),

    #[response(status = 404, description = "Feed entry not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
