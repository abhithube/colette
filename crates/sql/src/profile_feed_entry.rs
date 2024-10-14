use colette_core::feed_entry::Cursor;
use sea_query::{
    CaseStatement, ColumnDef, ColumnType, Expr, ForeignKey, ForeignKeyAction, Iden, Index,
    IndexCreateStatement, InsertStatement, JoinType, OnConflict, Order, Query, SelectStatement,
    Table, TableCreateStatement, UpdateStatement,
};
use uuid::Uuid;

use crate::{
    common::WithTimestamps, feed_entry::FeedEntry, profile::Profile, profile_feed::ProfileFeed,
    profile_feed_tag::ProfileFeedTag, smart_feed_filter::SmartFeedFilter, tag::Tag,
};

#[derive(sea_query::Iden)]
pub enum ProfileFeedEntry {
    Table,
    Id,
    HasRead,
    ProfileFeedId,
    FeedEntryId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(ProfileFeedEntry::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(ProfileFeedEntry::Id, id_type.clone())
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new_with_type(ProfileFeedEntry::HasRead, ColumnType::Boolean)
                .not_null()
                .default(false),
        )
        .col(ColumnDef::new_with_type(ProfileFeedEntry::ProfileFeedId, id_type.clone()).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId)
                .to(ProfileFeed::Table, ProfileFeed::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .col(
            ColumnDef::new_with_type(ProfileFeedEntry::FeedEntryId, ColumnType::Integer).not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId)
                .to(FeedEntry::Table, FeedEntry::Id)
                .on_delete(ForeignKeyAction::Restrict),
        )
        .col(ColumnDef::new_with_type(ProfileFeedEntry::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn create_profile_feed_id_feed_entry_id_index() -> IndexCreateStatement {
    Index::create()
        .name(format!(
            "{profile_feed_entry}_{profile_feed_id}_{feed_entry_id}_idx",
            profile_feed_entry = ProfileFeedEntry::Table.to_string(),
            profile_feed_id = ProfileFeedEntry::ProfileFeedId.to_string(),
            feed_entry_id = ProfileFeedEntry::FeedEntryId.to_string()
        ))
        .table(ProfileFeedEntry::Table)
        .if_not_exists()
        .col(ProfileFeedEntry::ProfileFeedId)
        .col(ProfileFeedEntry::FeedEntryId)
        .unique()
        .to_owned()
}

pub struct InsertMany {
    pub id: Uuid,
    pub feed_entry_id: i32,
}

#[allow(clippy::too_many_arguments)]
pub fn select(
    id: Option<Uuid>,
    profile_id: Uuid,
    feed_id: Option<Uuid>,
    has_read: Option<bool>,
    tags: Option<&[String]>,
    smart_feed_id: Option<Uuid>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
    smart_feed_case_statement: CaseStatement,
) -> SelectStatement {
    let mut query = Query::select()
        .columns([
            (ProfileFeedEntry::Table, ProfileFeedEntry::Id),
            (ProfileFeedEntry::Table, ProfileFeedEntry::HasRead),
            (ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId),
        ])
        .columns([
            (FeedEntry::Table, FeedEntry::Link),
            (FeedEntry::Table, FeedEntry::Title),
            (FeedEntry::Table, FeedEntry::PublishedAt),
            (FeedEntry::Table, FeedEntry::Description),
            (FeedEntry::Table, FeedEntry::Author),
            (FeedEntry::Table, FeedEntry::ThumbnailUrl),
        ])
        .from(ProfileFeedEntry::Table)
        .join(
            JoinType::InnerJoin,
            FeedEntry::Table,
            Expr::col((FeedEntry::Table, FeedEntry::Id)).eq(Expr::col((
                ProfileFeedEntry::Table,
                ProfileFeedEntry::FeedEntryId,
            ))),
        )
        .and_where(Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)).eq(profile_id))
        .and_where_option(
            id.map(|e| Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).eq(e)),
        )
        .and_where_option(
            feed_id.map(|e| {
                Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId)).eq(e)
            }),
        )
        .and_where_option(
            has_read.map(|e| Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::HasRead)).eq(e)),
        )
        .and_where_option(cursor.map(|e| {
            Expr::tuple([
                Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)).into(),
                Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::val(e.published_at).into(),
                Expr::val(e.id).into(),
            ]))
        }))
        .order_by((FeedEntry::Table, FeedEntry::PublishedAt), Order::Desc)
        .order_by((ProfileFeedEntry::Table, ProfileFeedEntry::Id), Order::Desc)
        .to_owned();

    if let Some(tags) = tags {
        query
            .join(
                JoinType::InnerJoin,
                ProfileFeedTag::Table,
                Expr::col((ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)).eq(Expr::col((
                    ProfileFeedEntry::Table,
                    ProfileFeedEntry::ProfileFeedId,
                ))),
            )
            .join(
                JoinType::InnerJoin,
                ProfileFeed::Table,
                Expr::col((Tag::Table, Tag::Id))
                    .eq(Expr::col((ProfileFeedTag::Table, ProfileFeedTag::TagId))),
            )
            .and_where(Expr::col((Tag::Table, Tag::Title)).is_in(tags));
    }

    if let Some(smart_feed_id) = smart_feed_id {
        query.join(
            JoinType::InnerJoin,
            SmartFeedFilter::Table,
            Expr::col((SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId))
                .eq(Expr::val(smart_feed_id))
                .and(smart_feed_case_statement.into()),
        );
    }

    if let Some(limit) = limit {
        query.limit(limit);
    }

    query
}

pub fn insert_many(data: Vec<InsertMany>, pf_id: Uuid, profile_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(ProfileFeedEntry::Table)
        .columns([
            ProfileFeedEntry::Id,
            ProfileFeedEntry::FeedEntryId,
            ProfileFeedEntry::ProfileFeedId,
            ProfileFeedEntry::ProfileId,
        ])
        .on_conflict(
            OnConflict::columns([
                ProfileFeedEntry::ProfileFeedId,
                ProfileFeedEntry::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .returning_col(FeedEntry::Id)
        .to_owned();

    for pfe in data {
        query.values_panic([
            pfe.id.into(),
            pfe.feed_entry_id.into(),
            pf_id.into(),
            profile_id.into(),
        ]);
    }

    query
}

pub fn update(id: Uuid, profile_id: Uuid, has_read: Option<bool>) -> UpdateStatement {
    let mut query = Query::update()
        .table(ProfileFeedEntry::Table)
        .value(ProfileFeedEntry::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::Id)).eq(id))
        .and_where(Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)).eq(profile_id))
        .to_owned();

    if let Some(has_read) = has_read {
        query.value(ProfileFeedEntry::HasRead, has_read);
    }

    query
}
