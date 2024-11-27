use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement};

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

pub fn select_by_link(link: String) -> SelectStatement {
    Query::select()
        .column(Bookmark::Id)
        .from(Bookmark::Table)
        .and_where(Expr::col(Bookmark::Link).eq(link))
        .to_owned()
}

pub fn insert(
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
) -> InsertStatement {
    Query::insert()
        .into_table(Bookmark::Table)
        .columns([
            Bookmark::Link,
            Bookmark::Title,
            Bookmark::ThumbnailUrl,
            Bookmark::PublishedAt,
            Bookmark::Author,
            Bookmark::UpdatedAt,
        ])
        .values_panic([
            link.into(),
            title.into(),
            thumbnail_url.into(),
            published_at.into(),
            author.into(),
            Expr::current_timestamp().into(),
        ])
        .on_conflict(
            OnConflict::column(Bookmark::Link)
                .update_columns([
                    Bookmark::Title,
                    Bookmark::ThumbnailUrl,
                    Bookmark::PublishedAt,
                    Bookmark::Author,
                    Bookmark::UpdatedAt,
                ])
                .to_owned(),
        )
        .returning_col(Bookmark::Id)
        .to_owned()
}
