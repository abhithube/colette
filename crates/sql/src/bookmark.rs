use chrono::{DateTime, Utc};
use sea_query::{Expr, InsertStatement, OnConflict, Query};

#[derive(sea_query::Iden)]
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
