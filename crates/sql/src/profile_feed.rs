use colette_core::feed::Cursor;
use sea_query::{
    Alias, ColumnDef, ColumnType, CommonTableExpression, DeleteStatement, Expr, ForeignKey,
    ForeignKeyAction, Func, Iden, Index, IndexCreateStatement, InsertStatement, JoinType,
    OnConflict, Query, SelectStatement, SimpleExpr, Table, TableCreateStatement, UpdateStatement,
    WithClause, WithQuery,
};
use uuid::Uuid;

use crate::{
    common::WithTimestamps, feed::Feed, profile::Profile, profile_feed_entry::ProfileFeedEntry,
    profile_feed_tag::ProfileFeedTag, tag::Tag,
};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum ProfileFeed {
    Table,
    Id,
    Title,
    Pinned,
    ProfileId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(ProfileFeed::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(ProfileFeed::Id, id_type.clone())
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new_with_type(
            ProfileFeed::Title,
            ColumnType::Text,
        ))
        .col(
            ColumnDef::new_with_type(ProfileFeed::Pinned, ColumnType::Boolean)
                .not_null()
                .default(false),
        )
        .col(ColumnDef::new_with_type(ProfileFeed::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeed::Table, ProfileFeed::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .col(ColumnDef::new_with_type(ProfileFeed::FeedId, ColumnType::Integer).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeed::Table, ProfileFeed::FeedId)
                .to(Feed::Table, Feed::Id)
                .on_delete(ForeignKeyAction::Restrict),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn create_profile_id_feed_id_index() -> IndexCreateStatement {
    Index::create()
        .name(format!(
            "{profile_feed}_{profile_id}_{feed_id}_idx",
            profile_feed = ProfileFeed::Table.to_string(),
            profile_id = ProfileFeed::ProfileId.to_string(),
            feed_id = ProfileFeed::FeedId.to_string()
        ))
        .table(ProfileFeed::Table)
        .if_not_exists()
        .col(ProfileFeed::ProfileId)
        .col(ProfileFeed::FeedId)
        .unique()
        .to_owned()
}

#[allow(clippy::too_many_arguments)]
pub fn select(
    id: Option<Uuid>,
    profile_id: Uuid,
    pinned: Option<bool>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
    jsonb_agg: SimpleExpr,
    tags_subquery: Option<SimpleExpr>,
) -> WithQuery {
    let unread_count = Alias::new("unread_count");
    let pf_id = Alias::new("pf_id");

    let unread_count_cte = Query::select()
        .expr_as(
            Expr::col((ProfileFeed::Table, ProfileFeed::Id)),
            pf_id.clone(),
        )
        .expr_as(
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).count(),
            unread_count.clone(),
        )
        .from(ProfileFeed::Table)
        .join(
            JoinType::InnerJoin,
            ProfileFeedEntry::Table,
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .group_by_col((ProfileFeed::Table, ProfileFeed::Id))
        .to_owned();

    let tags = Alias::new("tags");

    let json_tags_cte = Query::select()
        .expr_as(
            Expr::col((ProfileFeed::Table, ProfileFeed::Id)),
            pf_id.clone(),
        )
        .expr_as(jsonb_agg, tags.clone())
        .from(ProfileFeed::Table)
        .join(
            JoinType::InnerJoin,
            ProfileFeedTag::Table,
            Expr::col((ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .join(
            JoinType::InnerJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::Id))
                .eq(Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId))),
        )
        .group_by_col((ProfileFeed::Table, ProfileFeed::Id))
        .to_owned();

    let json_tags = Alias::new("json_tags");
    let unread_counts = Alias::new("unread_counts");

    let mut select = Query::select()
        .columns([
            (ProfileFeed::Table, ProfileFeed::Id),
            (ProfileFeed::Table, ProfileFeed::Title),
            (ProfileFeed::Table, ProfileFeed::Pinned),
        ])
        .columns([(Feed::Table, Feed::Link), (Feed::Table, Feed::Url)])
        .expr_as(
            Expr::col((Feed::Table, Feed::Title)),
            Alias::new("original_title"),
        )
        .column((json_tags.clone(), tags.clone()))
        .expr_as(
            Func::coalesce([
                Expr::col((unread_counts.clone(), unread_count.clone())).into(),
                Expr::val(0_i64).into(),
            ]),
            unread_count,
        )
        .from(ProfileFeed::Table)
        .join(
            JoinType::Join,
            Feed::Table,
            Expr::col((Feed::Table, Feed::Id))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::FeedId))),
        )
        .join(
            JoinType::LeftJoin,
            json_tags.clone(),
            Expr::col((json_tags.clone(), pf_id.clone()))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .join(
            JoinType::LeftJoin,
            unread_counts.clone(),
            Expr::col((unread_counts.clone(), pf_id.clone()))
                .eq(Expr::col((ProfileFeed::Table, ProfileFeed::Id))),
        )
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::ProfileId)).eq(profile_id))
        .and_where_option(id.map(|e| Expr::col((ProfileFeed::Table, ProfileFeed::Id)).eq(e)))
        .and_where_option(
            pinned.map(|e| Expr::col((ProfileFeed::Table, ProfileFeed::Pinned)).eq(e)),
        )
        .and_where_option(tags_subquery)
        .and_where_option(cursor.map(|e| {
            Expr::tuple([
                Func::coalesce([
                    Expr::col((ProfileFeed::Table, ProfileFeed::Title)).into(),
                    Expr::col((Feed::Table, Feed::Title)).into(),
                ])
                .into(),
                Expr::col((ProfileFeed::Table, ProfileFeed::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::val(e.title).into(),
                Expr::val(e.id).into(),
            ]))
        }))
        .to_owned();

    if let Some(limit) = limit {
        select.limit(limit);
    }

    select.with(
        WithClause::new()
            .cte(
                CommonTableExpression::new()
                    .query(json_tags_cte)
                    .table_name(json_tags)
                    .to_owned(),
            )
            .cte(
                CommonTableExpression::new()
                    .query(unread_count_cte)
                    .table_name(unread_counts)
                    .to_owned(),
            )
            .to_owned(),
    )
}

pub fn select_by_unique_index(profile_id: Uuid, feed_id: i32) -> SelectStatement {
    Query::select()
        .column(ProfileFeed::Id)
        .from(ProfileFeed::Table)
        .and_where(Expr::col(ProfileFeed::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileFeed::FeedId).eq(feed_id))
        .to_owned()
}

pub fn insert(id: Uuid, pinned: Option<bool>, feed_id: i32, profile_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(ProfileFeed::Table)
        .columns([
            ProfileFeed::Id,
            ProfileFeed::Pinned,
            ProfileFeed::FeedId,
            ProfileFeed::ProfileId,
        ])
        .values_panic([
            id.into(),
            pinned.unwrap_or_default().into(),
            feed_id.into(),
            profile_id.into(),
        ])
        .on_conflict(
            OnConflict::columns([ProfileFeed::ProfileId, ProfileFeed::FeedId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned()
}

pub fn update(
    id: Uuid,
    profile_id: Uuid,
    title: Option<Option<String>>,
    pinned: Option<bool>,
) -> UpdateStatement {
    let mut query = Query::update()
        .table(ProfileFeed::Table)
        .value(ProfileFeed::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::Id)).eq(id))
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::ProfileId)).eq(profile_id))
        .to_owned();

    if let Some(title) = title {
        query.value(ProfileFeed::Title, title);
    }
    if let Some(pinned) = pinned {
        query.value(ProfileFeed::Pinned, pinned);
    }

    query
}

pub fn delete(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileFeed::Table)
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::Id)).eq(id))
        .and_where(Expr::col((ProfileFeed::Table, ProfileFeed::ProfileId)).eq(profile_id))
        .to_owned()
}
