use std::sync::Arc;

use colette_core::tag::TagService;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::Paginated;

mod create_tag;
mod delete_tag;
mod get_tag;
mod list_tags;
mod update_tag;

#[derive(Clone, axum::extract::FromRef)]
pub struct TagState {
    service: Arc<TagService>,
}

impl TagState {
    pub fn new(service: Arc<TagService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Tag, Paginated<Tag>, create_tag::TagCreate, update_tag::TagUpdate)))]
pub struct TagApi;

impl TagApi {
    pub fn router() -> OpenApiRouter<TagState> {
        OpenApiRouter::with_openapi(TagApi::openapi())
            .routes(routes!(list_tags::handler, create_tag::handler))
            .routes(routes!(
                get_tag::handler,
                update_tag::handler,
                delete_tag::handler
            ))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
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
            bookmark_count: value.bookmark_count,
            feed_count: value.feed_count,
        }
    }
}
