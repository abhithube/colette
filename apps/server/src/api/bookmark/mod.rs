use chrono::{DateTime, Utc};
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, tag::Tag};
use crate::api::common::Paginated;

mod create_bookmark;
mod delete_bookmark;
mod get_bookmark;
mod list_bookmarks;
mod scrape_bookmark;
mod update_bookmark;

pub const BOOKMARKS_TAG: &str = "Bookmarks";

#[derive(OpenApi)]
#[openapi(components(schemas(Bookmark, Paginated<Bookmark>, create_bookmark::BookmarkCreate, update_bookmark::BookmarkUpdate, scrape_bookmark::BookmarkScrape, scrape_bookmark::BookmarkScraped)))]
pub struct BookmarkApi;

impl BookmarkApi {
    pub fn router() -> OpenApiRouter<ApiState> {
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

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

impl From<(colette_core::Bookmark, Url)> for Bookmark {
    fn from((value, bucket_url): (colette_core::Bookmark, Url)) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_url: value.archived_path.map(|e| bucket_url.join(&e).unwrap()),
            created_at: value.created_at,
            updated_at: value.updated_at,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
        }
    }
}
