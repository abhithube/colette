use chrono::{DateTime, Utc};
use sea_query::{DeleteStatement, Expr, InsertStatement, OnConflict, Query, SelectStatement};

use crate::profile_feed_entry::ProfileFeedEntry;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
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

pub fn insert_many(data: Vec<InsertMany>, feed_id: i32) -> InsertStatement {
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
        ])
        .on_conflict(
            OnConflict::columns([FeedEntry::FeedId, FeedEntry::Link])
                .update_columns([
                    FeedEntry::Title,
                    FeedEntry::PublishedAt,
                    FeedEntry::Description,
                    FeedEntry::Author,
                    FeedEntry::ThumbnailUrl,
                    FeedEntry::FeedId,
                ])
                .to_owned(),
        )
        .returning_col(FeedEntry::Id)
        .to_owned();

    for fe in data {
        query.values_panic([
            fe.link.into(),
            fe.title.into(),
            fe.published_at.into(),
            fe.description.into(),
            fe.author.into(),
            fe.thumbnail_url.into(),
            feed_id.into(),
        ]);
    }

    query
}

pub fn delete_many() -> DeleteStatement {
    let subquery = Query::select()
        .from(ProfileFeedEntry::Table)
        .and_where(
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId))
                .equals((FeedEntry::Table, FeedEntry::Id)),
        )
        .to_owned();

    Query::delete()
        .from_table(FeedEntry::Table)
        .and_where(Expr::exists(subquery).not())
        .to_owned()
}
