use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::tag::TagDto;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated};

mod create_tag;
mod delete_tag;
mod get_tag;
mod list_tags;
mod update_tag;

const TAGS_TAG: &str = "Tags";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Tag, Paginated<Tag>, create_tag::TagCreate, update_tag::TagUpdate)),
    paths(list_tags::handler, create_tag::handler, get_tag::handler, update_tag::handler, delete_tag::handler)
)]
pub(crate) struct TagApi;

impl TagApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_tags::handler))
            .route("/", routing::post(create_tag::handler))
            .route("/{id}", routing::get(get_tag::handler))
            .route("/{id}", routing::patch(update_tag::handler))
            .route("/{id}", routing::delete(delete_tag::handler))
    }
}

/// Tag that can be attached to subscriptions and bookmarks
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tag {
    /// Unique identifier of the tag
    id: Uuid,
    /// Human-readable name of the tag, unique per user
    title: String,
    /// Timestamp at which the tag was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the tag was last modified
    updated_at: DateTime<Utc>,
}

impl From<TagDto> for Tag {
    fn from(value: TagDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
