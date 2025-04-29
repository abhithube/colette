use chrono::{DateTime, Utc};
use colette_core::bookmark;
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{
    ApiState,
    common::{DateOp, TextOp},
    tag::Tag,
};
use crate::common::Paginated;

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
#[openapi(components(schemas(Bookmark, BookmarkDetails, Paginated<BookmarkDetails>, create_bookmark::BookmarkCreate, update_bookmark::BookmarkUpdate, link_bookmark_tags::LinkBookmarkTags, scrape_bookmark::BookmarkScrape, scrape_bookmark::BookmarkScraped, BookmarkFilter, BookmarkTextField, BookmarkDateField)))]
pub(crate) struct BookmarkApi;

impl BookmarkApi {
    pub(crate) fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(BookmarkApi::openapi())
            .routes(routes!(list_bookmarks::handler, create_bookmark::handler))
            .routes(routes!(
                get_bookmark::handler,
                update_bookmark::handler,
                delete_bookmark::handler
            ))
            .routes(routes!(link_bookmark_tags::handler))
            .routes(routes!(scrape_bookmark::handler))
            .routes(routes!(import_bookmarks::handler))
            .routes(routes!(export_bookmarks::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Bookmark {
    id: Uuid,
    link: Url,
    title: String,
    #[schema(required)]
    thumbnail_url: Option<Url>,
    #[schema(required)]
    published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    author: Option<String>,
    #[schema(required)]
    archived_url: Option<Url>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
struct BookmarkDetails {
    bookmark: Bookmark,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<Tag>>,
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
        }
    }
}

impl From<(colette_core::Bookmark, Url)> for BookmarkDetails {
    fn from((value, bucket_url): (colette_core::Bookmark, Url)) -> Self {
        let tags = value
            .tags
            .clone()
            .map(|e| e.into_iter().map(Tag::from).collect());

        Self {
            bookmark: (value, bucket_url).into(),
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
