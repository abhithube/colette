use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, subscription_entry::SubscriptionEntryFilter};
use crate::common::Paginated;

mod create_stream;
mod delete_stream;
mod get_stream;
mod list_streams;
mod update_stream;

const STREAMS_TAG: &str = "Streams";

#[derive(OpenApi)]
#[openapi(components(schemas(Stream, Paginated<Stream>, create_stream::StreamCreate, update_stream::StreamUpdate)))]
pub(crate) struct StreamApi;

impl StreamApi {
    pub(crate) fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(StreamApi::openapi())
            .routes(routes!(list_streams::handler, create_stream::handler))
            .routes(routes!(
                get_stream::handler,
                update_stream::handler,
                delete_stream::handler
            ))
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
