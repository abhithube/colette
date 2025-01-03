use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement};

#[allow(dead_code)]
pub enum FeedEntry {
    Table,
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for FeedEntry {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "feed_entries",
                Self::Id => "id",
                Self::Link => "link",
                Self::Title => "title",
                Self::PublishedAt => "published_at",
                Self::Description => "description",
                Self::Author => "author",
                Self::ThumbnailUrl => "thumbnail_url",
                Self::FeedId => "feed_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct InsertMany {
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub fn select_many_by_feed_id(feed_id: i32) -> SelectStatement {
    Query::select()
        .column(FeedEntry::Id)
        .from(FeedEntry::Table)
        .and_where(Expr::col(FeedEntry::FeedId).eq(feed_id))
        .to_owned()
}

pub fn insert_many(data: &[InsertMany], feed_id: i32) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(FeedEntry::Table)
        .columns([
            FeedEntry::Link,
            FeedEntry::Title,
            FeedEntry::PublishedAt,
            FeedEntry::Description,
            FeedEntry::Author,
            FeedEntry::ThumbnailUrl,
            FeedEntry::FeedId,
            FeedEntry::UpdatedAt,
        ])
        .on_conflict(
            OnConflict::columns([FeedEntry::FeedId, FeedEntry::Link])
                .update_columns([
                    FeedEntry::Title,
                    FeedEntry::PublishedAt,
                    FeedEntry::Description,
                    FeedEntry::Author,
                    FeedEntry::ThumbnailUrl,
                    FeedEntry::UpdatedAt,
                ])
                .to_owned(),
        )
        .to_owned();

    for fe in data {
        query.values_panic([
            (*fe.link).into(),
            (*fe.title).into(),
            fe.published_at.into(),
            fe.description.as_deref().into(),
            fe.author.as_deref().into(),
            fe.thumbnail_url.as_deref().into(),
            feed_id.into(),
            Expr::current_timestamp().into(),
        ]);
    }

    query
}
