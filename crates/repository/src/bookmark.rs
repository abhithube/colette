use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::Iden;
use sqlx::PgExecutor;
use uuid::Uuid;

#[allow(dead_code)]
pub enum Bookmark {
    Table,
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Bookmark {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "bookmarks",
                Self::Id => "id",
                Self::Link => "link",
                Self::Title => "title",
                Self::ThumbnailUrl => "thumbnail_url",
                Self::PublishedAt => "published_at",
                Self::Author => "author",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub async fn select_by_link<'a>(ex: impl PgExecutor<'a>, link: String) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "SELECT id
FROM bookmarks
WHERE link = $1",
        link
    )
    .fetch_one(ex)
    .await
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "INSERT INTO bookmarks (link, title, thumbnail_url, published_at, author, updated_at)
VALUES ($1, $2, $3, $4, $5, now())
ON CONFLICT (link) DO UPDATE SET
    title = excluded.title,
    thumbnail_url = excluded.thumbnail_url,
    published_at = excluded.published_at,
    author = excluded.author,
    updated_at = excluded.updated_at
RETURNING id",
        link,
        title,
        thumbnail_url,
        published_at,
        author
    )
    .fetch_one(ex)
    .await
}
