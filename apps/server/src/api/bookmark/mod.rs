use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_core::bookmark::BookmarkService;
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::tag::Tag;
use crate::api::common::Paginated;

mod create_bookmark;
mod delete_bookmark;
mod get_bookmark;
mod list_bookmarks;
mod scrape_bookmark;
mod update_bookmark;

#[derive(Clone, axum::extract::FromRef)]
pub struct BookmarkState {
    service: Arc<BookmarkService>,
}

impl BookmarkState {
    pub fn new(service: Arc<BookmarkService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Bookmark, Paginated<Bookmark>, create_bookmark::BookmarkCreate, update_bookmark::BookmarkUpdate, scrape_bookmark::BookmarkScrape, scrape_bookmark::BookmarkScraped)))]
pub struct BookmarkApi;

impl BookmarkApi {
    pub fn router() -> OpenApiRouter<BookmarkState> {
        OpenApiRouter::with_openapi(BookmarkApi::openapi())
            .routes(routes!(list_bookmarks::handler, create_bookmark::handler))
            .routes(routes!(
                get_bookmark::handler,
                update_bookmark::handler,
                delete_bookmark::handler
            ))
            .routes(routes!(scrape_bookmark::handler))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    #[schema(required)]
    pub thumbnail_url: Option<Url>,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub author: Option<String>,
    #[schema(required)]
    pub archived_url: Option<Url>,
    #[schema(required)]
    pub folder_id: Option<Uuid>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

impl From<colette_core::Bookmark> for Bookmark {
    fn from(value: colette_core::Bookmark) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_url: value.archived_url,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
        }
    }
}
