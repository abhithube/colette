use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::bookmark;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    ApiState,
    common::{DateOp, TextOp},
    pagination::Paginated,
    tag::Tag,
};

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
    components(schemas(Bookmark, BookmarkDetails, Paginated<BookmarkDetails>, create_bookmark::BookmarkCreate, update_bookmark::BookmarkUpdate, link_bookmark_tags::LinkBookmarkTags, scrape_bookmark::BookmarkScrape, scrape_bookmark::BookmarkScraped, BookmarkFilter, BookmarkTextField, BookmarkDateField)),
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
    /// Timestamp at which the bookmark was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the bookmark was modified
    updated_at: DateTime<Utc>,
}

/// Extended details of a bookmark
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
struct BookmarkDetails {
    /// Bookmark itself, always present
    bookmark: Bookmark,
    #[schema(nullable = false)]
    /// Linked tags, present if requested
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<Tag>>,
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
            archived_path: value.archived_path,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<colette_core::Bookmark> for BookmarkDetails {
    fn from(value: colette_core::Bookmark) -> Self {
        let tags = value
            .tags
            .clone()
            .map(|e| e.into_iter().map(Tag::from).collect());

        Self {
            bookmark: value.into(),
            tags,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(no_recursion)]
pub(crate) enum BookmarkFilter {
    Text {
        field: BookmarkTextField,
        op: TextOp,
    },
    Date {
        field: BookmarkDateField,
        op: DateOp,
    },

    And(Vec<BookmarkFilter>),
    Or(Vec<BookmarkFilter>),
    Not(Box<BookmarkFilter>),
}

impl From<BookmarkFilter> for bookmark::BookmarkFilter {
    fn from(value: BookmarkFilter) -> Self {
        match value {
            BookmarkFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            BookmarkFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            BookmarkFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            BookmarkFilter::Or(filters) => Self::Or(filters.into_iter().map(Into::into).collect()),
            BookmarkFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

impl From<bookmark::BookmarkFilter> for BookmarkFilter {
    fn from(value: bookmark::BookmarkFilter) -> Self {
        match value {
            bookmark::BookmarkFilter::Text { field, op } => Self::Text {
                field: field.into(),
                op: op.into(),
            },
            bookmark::BookmarkFilter::Date { field, op } => Self::Date {
                field: field.into(),
                op: op.into(),
            },
            bookmark::BookmarkFilter::And(filters) => {
                Self::And(filters.into_iter().map(Into::into).collect())
            }
            bookmark::BookmarkFilter::Or(filters) => {
                Self::Or(filters.into_iter().map(Into::into).collect())
            }
            bookmark::BookmarkFilter::Not(filter) => Self::Not(Box::new((*filter).into())),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BookmarkTextField {
    Link,
    Title,
    Author,
    Tag,
}

impl From<BookmarkTextField> for bookmark::BookmarkTextField {
    fn from(value: BookmarkTextField) -> Self {
        match value {
            BookmarkTextField::Title => Self::Title,
            BookmarkTextField::Link => Self::Link,
            BookmarkTextField::Author => Self::Author,
            BookmarkTextField::Tag => Self::Tag,
        }
    }
}

impl From<bookmark::BookmarkTextField> for BookmarkTextField {
    fn from(value: bookmark::BookmarkTextField) -> Self {
        match value {
            bookmark::BookmarkTextField::Title => Self::Title,
            bookmark::BookmarkTextField::Link => Self::Link,
            bookmark::BookmarkTextField::Author => Self::Author,
            bookmark::BookmarkTextField::Tag => Self::Tag,
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) enum BookmarkDateField {
    PublishedAt,
    CreatedAt,
    UpdatedAt,
}

impl From<BookmarkDateField> for bookmark::BookmarkDateField {
    fn from(value: BookmarkDateField) -> Self {
        match value {
            BookmarkDateField::PublishedAt => Self::PublishedAt,
            BookmarkDateField::CreatedAt => Self::CreatedAt,
            BookmarkDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}

impl From<bookmark::BookmarkDateField> for BookmarkDateField {
    fn from(value: bookmark::BookmarkDateField) -> Self {
        match value {
            bookmark::BookmarkDateField::PublishedAt => Self::PublishedAt,
            bookmark::BookmarkDateField::CreatedAt => Self::CreatedAt,
            bookmark::BookmarkDateField::UpdatedAt => Self::UpdatedAt,
        }
    }
}
