use colette_core::smart_feed::Cursor;
use sea_query::{
    Alias, CommonTableExpression, Expr, Func, JoinType, Order, PostgresQueryBuilder, Query,
    WithClause,
};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

use crate::{
    feed_entry::FeedEntry,
    profile_feed_entry::ProfileFeedEntry,
    smart_feed_filter::{build_case_statement, SmartFeedFilter},
};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum SmartFeed {
    Table,
    Id,
    Title,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct SmartFeedSelect {
    id: Uuid,
    title: String,
    unread_count: i64,
}

impl From<SmartFeedSelect> for colette_core::SmartFeed {
    fn from(value: SmartFeedSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            unread_count: Some(value.unread_count),
        }
    }
}

pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::SmartFeed>> {
    let sf_id = Alias::new("sf_id");
    let unread_count = Alias::new("unread_count");

    let unread_counts_cte = Query::select()
        .expr_as(Expr::col((SmartFeed::Table, SmartFeed::Id)), sf_id.clone())
        .expr_as(
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).count(),
            unread_count.clone(),
        )
        .from(SmartFeed::Table)
        .join(
            JoinType::LeftJoin,
            ProfileFeedEntry::Table,
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId))
                .eq(Expr::col((SmartFeed::Table, SmartFeed::ProfileId))),
        )
        .join(
            JoinType::LeftJoin,
            FeedEntry::Table,
            Expr::col((FeedEntry::Table, FeedEntry::Id)).eq(Expr::col((
                ProfileFeedEntry::Table,
                ProfileFeedEntry::FeedEntryId,
            ))),
        )
        .join(
            JoinType::Join,
            SmartFeedFilter::Table,
            Expr::col((SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId))
                .eq(Expr::col((SmartFeed::Table, SmartFeed::Id)))
                .and(build_case_statement().into()),
        )
        .and_where(Expr::col((SmartFeed::Table, SmartFeed::ProfileId)).eq(profile_id))
        .group_by_col((SmartFeed::Table, SmartFeed::Id))
        .to_owned();

    let unread_counts = Alias::new("unread_counts");

    let mut select = Query::select()
        .columns([
            (SmartFeed::Table, SmartFeed::Id),
            (SmartFeed::Table, SmartFeed::Title),
        ])
        .expr_as(
            Func::coalesce([
                Expr::col((unread_counts.clone(), unread_count.clone())).into(),
                Expr::val(0).into(),
            ]),
            unread_count,
        )
        .from(SmartFeed::Table)
        .join(
            JoinType::LeftJoin,
            unread_counts.clone(),
            Expr::col((unread_counts.clone(), sf_id))
                .eq(Expr::col((SmartFeed::Table, SmartFeed::Id))),
        )
        .and_where(Expr::col((SmartFeed::Table, SmartFeed::ProfileId)).eq(profile_id))
        .and_where_option(id.map(|e| Expr::col((SmartFeed::Table, SmartFeed::Id)).eq(e)))
        .and_where_option(
            cursor.map(|e| Expr::col((SmartFeed::Table, SmartFeed::Title)).gt(e.title)),
        )
        .order_by((SmartFeed::Table, SmartFeed::Title), Order::Asc)
        .to_owned();

    if let Some(limit) = limit {
        select.limit(limit);
    }

    let query = select.with(
        WithClause::new()
            .cte(
                CommonTableExpression::new()
                    .query(unread_counts_cte)
                    .table_name(unread_counts)
                    .to_owned(),
            )
            .to_owned(),
    );

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, SmartFeedSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}
