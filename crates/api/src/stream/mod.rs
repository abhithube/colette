use axum::{Router, routing};
use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated, subscription_entry::SubscriptionEntryFilter};

mod create_stream;
mod delete_stream;
mod get_stream;
mod list_streams;
mod update_stream;

const STREAMS_TAG: &str = "Streams";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Stream, Paginated<Stream>, create_stream::StreamCreate, update_stream::StreamUpdate)),
    paths(list_streams::handler, create_stream::handler, get_stream::handler, update_stream::handler, delete_stream::handler)
)]
pub(crate) struct StreamApi;

impl StreamApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_streams::handler))
            .route("/", routing::post(create_stream::handler))
            .route("/{id}", routing::get(get_stream::handler))
            .route("/{id}", routing::patch(update_stream::handler))
            .route("/{id}", routing::delete(delete_stream::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Stream {
    id: Uuid,
    title: String,
    filter: SubscriptionEntryFilter,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<colette_core::Stream> for Stream {
    fn from(value: colette_core::Stream) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter.into(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
