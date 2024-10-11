use colette_core::smart_feed::Cursor;
use sea_query::{
    Alias, CaseStatement, ColumnDef, ColumnType, CommonTableExpression, DeleteStatement, Expr,
    ForeignKey, ForeignKeyAction, Func, Iden, Index, IndexCreateStatement, InsertStatement,
    JoinType, Order, Query, Table, TableCreateStatement, UpdateStatement, WithClause, WithQuery,
};
use uuid::Uuid;

use crate::{
    common::WithTimestamps, feed_entry::FeedEntry, profile::Profile,
    profile_feed_entry::ProfileFeedEntry, smart_feed_filter::SmartFeedFilter,
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

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(SmartFeed::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(SmartFeed::Id, id_type.clone())
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new_with_type(SmartFeed::Title, ColumnType::Text).not_null())
        .col(ColumnDef::new_with_type(SmartFeed::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(SmartFeed::Table, SmartFeed::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn create_profile_id_title_index() -> IndexCreateStatement {
    Index::create()
        .name(format!(
            "{smart_feed}_{profile_id}_{title}_idx",
            smart_feed = SmartFeed::Table.to_string(),
            profile_id = SmartFeed::ProfileId.to_string(),
            title = SmartFeed::Title.to_string()
        ))
        .table(SmartFeed::Table)
        .if_not_exists()
        .col(SmartFeed::ProfileId)
        .col(SmartFeed::Title)
        .unique()
        .to_owned()
}

pub fn select(
    id: Option<Uuid>,
    profile_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<u64>,
    smart_feed_case_statement: CaseStatement,
) -> WithQuery {
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
                .and(smart_feed_case_statement.into()),
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
                Expr::val(0_i64).into(),
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

    select.with(
        WithClause::new()
            .cte(
                CommonTableExpression::new()
                    .query(unread_counts_cte)
                    .table_name(unread_counts)
                    .to_owned(),
            )
            .to_owned(),
    )
}

pub fn insert(id: Uuid, title: String, profile_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(SmartFeed::Table)
        .columns([SmartFeed::Id, SmartFeed::Title, SmartFeed::ProfileId])
        .values_panic([id.into(), title.into(), profile_id.into()])
        .to_owned()
}

pub fn update(id: Uuid, profile_id: Uuid, title: Option<String>) -> UpdateStatement {
    let mut query = Query::update()
        .table(SmartFeed::Table)
        .value(SmartFeed::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(SmartFeed::Id).eq(id))
        .and_where(Expr::col(SmartFeed::ProfileId).eq(profile_id))
        .to_owned();

    if let Some(title) = title {
        query.value(SmartFeed::Title, title);
    }

    query
}

pub fn delete(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(SmartFeed::Table)
        .and_where(Expr::col((SmartFeed::Table, SmartFeed::Id)).eq(id))
        .and_where(Expr::col((SmartFeed::Table, SmartFeed::ProfileId)).eq(profile_id))
        .to_owned()
}
