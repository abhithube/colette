use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::ApiState;
use crate::api::common::Paginated;

mod create_tag;
mod delete_tag;
mod get_tag;
mod list_tags;
mod update_tag;

pub const TAGS_TAG: &str = "Tags";

#[derive(OpenApi)]
#[openapi(components(schemas(Tag, Paginated<Tag>, create_tag::TagCreate, update_tag::TagUpdate)))]
pub struct TagApi;

impl TagApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(TagApi::openapi())
            .routes(routes!(list_tags::handler, create_tag::handler))
            .routes(routes!(
                get_tag::handler,
                update_tag::handler,
                delete_tag::handler
            ))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bookmark_count: Option<i64>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed_count: Option<i64>,
}

impl From<colette_core::Tag> for Tag {
    fn from(value: colette_core::Tag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
            bookmark_count: value.bookmark_count,
            feed_count: value.feed_count,
        }
    }
}
