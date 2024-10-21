use std::fmt::Write;

use colette_core::feed_entry::Cursor;
use sea_query::{
    Alias, Asterisk, CaseStatement, CommonTableExpression, Expr, Iden, InsertStatement, JoinType,
    OnConflict, Order, Query, SelectStatement, UpdateStatement, WithClause, WithQuery,
};
use uuid::Uuid;

use crate::{
    feed_entry::FeedEntry, profile_feed::ProfileFeed, profile_feed_tag::ProfileFeedTag,
    smart_feed_filter::SmartFeedFilter, tag::Tag,
};

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

impl Iden for ProfileFeedEntry {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "profile_feed_entries",
                Self::Id => "id",
                Self::HasRead => "has_read",
                Self::ProfileFeedId => "profile_feed_id",
                Self::FeedEntryId => "feed_entry_id",
                Self::ProfileId => "profile_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
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

pub fn insert_many_for_all_profiles(data: Vec<InsertMany>, feed_id: i32) -> WithQuery {
    let input = Alias::new("input");

    let input_cte = Query::select()
        .expr(Expr::col(Asterisk))
        .from_values(
            data.into_iter()
                .map(|e| (e.id, e.feed_entry_id, feed_id))
                .collect::<Vec<_>>(),
            Alias::new("row"),
        )
        .to_owned();

    let with_clause = WithClause::new()
        .cte(
            CommonTableExpression::new()
                .query(input_cte)
                .columns([ProfileFeedEntry::Id, ProfileFeedEntry::FeedEntryId])
                .column(ProfileFeed::FeedId)
                .table_name(input.clone())
                .to_owned(),
        )
        .to_owned();

    let insert = Query::insert()
        .into_table(ProfileFeedEntry::Table)
        .columns([
            ProfileFeedEntry::Id,
            ProfileFeedEntry::FeedEntryId,
            ProfileFeedEntry::ProfileFeedId,
            ProfileFeedEntry::ProfileId,
        ])
        .select_from(
            Query::select()
                .columns([
                    (input.clone(), ProfileFeedEntry::Id),
                    (input.clone(), ProfileFeedEntry::FeedEntryId),
                ])
                .columns([
                    (ProfileFeed::Table, ProfileFeed::Id),
                    (ProfileFeed::Table, ProfileFeed::ProfileId),
                ])
                .from(ProfileFeed::Table)
                .join(
                    JoinType::InnerJoin,
                    input.clone(),
                    Expr::col((input, ProfileFeed::FeedId))
                        .eq(Expr::col((ProfileFeed::Table, ProfileFeed::FeedId))),
                )
                .to_owned(),
        )
        .unwrap()
        .on_conflict(
            OnConflict::columns([
                ProfileFeedEntry::ProfileFeedId,
                ProfileFeedEntry::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .to_owned();

    insert.with(with_clause)
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
