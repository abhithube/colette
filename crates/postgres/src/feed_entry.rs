use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::chrono::{DateTime, Utc},
    PgExecutor,
};

use crate::profile_feed_entry::ProfileFeedEntry;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum FeedEntry {
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

pub async fn select_many_by_feed_id(
    executor: impl PgExecutor<'_>,
    feed_id: i32,
) -> sqlx::Result<Vec<i32>> {
    let query = Query::select()
        .column(FeedEntry::Id)
        .from(FeedEntry::Table)
        .and_where(Expr::col(FeedEntry::FeedId).eq(feed_id))
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
        .fetch_all(executor)
        .await
}

pub struct InsertMany {
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    feed_id: i32,
) -> sqlx::Result<()> {
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

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many(executor: impl PgExecutor<'_>) -> sqlx::Result<u64> {
    let subquery = Query::select()
        .from(ProfileFeedEntry::Table)
        .and_where(
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId))
                .equals((FeedEntry::Table, FeedEntry::Id)),
        )
        .to_owned();

    let query = Query::delete()
        .from_table(FeedEntry::Table)
        .and_where(Expr::exists(subquery).not())
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result.rows_affected())
}
