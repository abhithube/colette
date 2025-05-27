use axum::{Router, routing};
use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use uuid::Uuid;

use super::ApiState;
use crate::common::Paginated;

mod create_tag;
mod delete_tag;
mod get_tag;
mod list_tags;
mod update_tag;

const TAGS_TAG: &str = "Tags";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Tag, TagDetails, Paginated<TagDetails>, create_tag::TagCreate, update_tag::TagUpdate)),
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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Tag {
    id: Uuid,
    title: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct TagDetails {
    tag: Tag,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed_count: Option<i64>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bookmark_count: Option<i64>,
}

impl From<colette_core::Tag> for Tag {
    fn from(value: colette_core::Tag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<colette_core::Tag> for TagDetails {
    fn from(value: colette_core::Tag) -> Self {
        let feed_count = value.feed_count;
        let bookmark_count = value.bookmark_count;

        Self {
            tag: value.into(),
            feed_count,
            bookmark_count,
        }
    }
}
