use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_handler::BookmarkDto;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated, tag::Tag};

mod create_bookmark;
mod delete_bookmark;
mod export_bookmarks;
mod get_bookmark;
mod import_bookmarks;
mod link_bookmark_tags;
mod list_bookmarks;
mod scrape_bookmark;
mod update_bookmark;

const BOOKMARKS_TAG: &str = "Bookmarks";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Bookmark, Paginated<Bookmark>, create_bookmark::BookmarkCreate, update_bookmark::BookmarkUpdate, link_bookmark_tags::LinkBookmarkTags, scrape_bookmark::BookmarkScrape, scrape_bookmark::BookmarkScraped)),
    paths(list_bookmarks::handler, create_bookmark::handler, get_bookmark::handler, update_bookmark::handler, delete_bookmark::handler, link_bookmark_tags::handler, scrape_bookmark::handler, import_bookmarks::handler, export_bookmarks::handler)
)]
pub(crate) struct BookmarkApi;

impl BookmarkApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_bookmarks::handler))
            .route("/", routing::post(create_bookmark::handler))
            .route("/{id}", routing::get(get_bookmark::handler))
            .route("/{id}", routing::patch(update_bookmark::handler))
            .route("/{id}", routing::delete(delete_bookmark::handler))
            .route("/{id}/linkTags", routing::post(link_bookmark_tags::handler))
            .route("/scrape", routing::post(scrape_bookmark::handler))
            .route("/import", routing::post(import_bookmarks::handler))
            .route("/export", routing::post(export_bookmarks::handler))
    }
}

/// Bookmark to a webpage
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Bookmark {
    /// Unique identifier of the bookmark
    id: Uuid,
    /// URL of the webpage the bookmark links to
    link: Url,
    /// Title of the bookmark
    title: String,
    /// Thumbnail URL of the bookmark
    #[schema(required)]
    thumbnail_url: Option<Url>,
    /// Timestamp at which the bookmark was published
    #[schema(required)]
    published_at: Option<DateTime<Utc>>,
    /// Author of the bookmark
    #[schema(required)]
    author: Option<String>,
    /// Storage path of the archived version of the bookmark's thumbnail
    #[schema(required)]
    archived_path: Option<String>,
    /// Linked tags
    tags: Vec<Tag>,
    /// Timestamp at which the bookmark was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the bookmark was modified
    updated_at: DateTime<Utc>,
}

impl From<BookmarkDto> for Bookmark {
    fn from(value: BookmarkDto) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            tags: value.tags.into_iter().map(Into::into).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
