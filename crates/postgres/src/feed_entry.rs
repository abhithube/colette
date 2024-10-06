use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgExecutor;

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
