use std::fmt::Write;

use colette_core::feed_entry::Cursor;
use sea_query::{
    Alias, Asterisk, CaseStatement, CommonTableExpression, Expr, Iden, IntoValueTuple, JoinType,
    OnConflict, Order, Query, SelectStatement, UpdateStatement, WithQuery,
};
use uuid::Uuid;

use crate::{
    feed_entry::FeedEntry, smart_feed_filter::SmartFeedFilter, tag::Tag, user_feed::UserFeed,
    user_feed_tag::UserFeedTag,
};

#[allow(dead_code)]
pub enum UserFeedEntry {
    Table,
    Id,
    HasRead,
    UserFeedId,
    FeedEntryId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for UserFeedEntry {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "user_feed_entries",
                Self::Id => "id",
                Self::HasRead => "has_read",
                Self::UserFeedId => "user_feed_id",
                Self::FeedEntryId => "feed_entry_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct InsertMany {
    pub id: Option<Uuid>,
    pub feed_entry_id: Uuid,
}

#[allow(clippy::too_many_arguments)]
pub fn select(
    id: Option<Uuid>,
    user_id: Uuid,
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
            (UserFeedEntry::Table, UserFeedEntry::Id),
            (UserFeedEntry::Table, UserFeedEntry::HasRead),
            (UserFeedEntry::Table, UserFeedEntry::UserFeedId),
        ])
        .columns([
            (FeedEntry::Table, FeedEntry::Link),
            (FeedEntry::Table, FeedEntry::Title),
            (FeedEntry::Table, FeedEntry::PublishedAt),
            (FeedEntry::Table, FeedEntry::Description),
            (FeedEntry::Table, FeedEntry::Author),
            (FeedEntry::Table, FeedEntry::ThumbnailUrl),
        ])
        .from(UserFeedEntry::Table)
        .join(
            JoinType::InnerJoin,
            FeedEntry::Table,
            Expr::col((FeedEntry::Table, FeedEntry::Id)).eq(Expr::col((
                UserFeedEntry::Table,
                UserFeedEntry::FeedEntryId,
            ))),
        )
        .and_where(Expr::col((UserFeedEntry::Table, UserFeedEntry::UserId)).eq(user_id))
        .and_where_option(id.map(|e| Expr::col((UserFeedEntry::Table, UserFeedEntry::Id)).eq(e)))
        .and_where_option(
            feed_id.map(|e| Expr::col((UserFeedEntry::Table, UserFeedEntry::UserFeedId)).eq(e)),
        )
        .and_where_option(
            has_read.map(|e| Expr::col((UserFeedEntry::Table, UserFeedEntry::HasRead)).eq(e)),
        )
        .and_where_option(cursor.map(|e| {
            Expr::tuple([
                Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)).into(),
                Expr::col((UserFeedEntry::Table, UserFeedEntry::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::val(e.published_at).into(),
                Expr::val(e.id).into(),
            ]))
        }))
        .order_by((FeedEntry::Table, FeedEntry::PublishedAt), Order::Desc)
        .order_by((UserFeedEntry::Table, UserFeedEntry::Id), Order::Desc)
        .to_owned();

    if let Some(tags) = tags {
        query
            .join(
                JoinType::InnerJoin,
                UserFeedTag::Table,
                Expr::col((UserFeedTag::Table, UserFeedTag::UserFeedId))
                    .eq(Expr::col((UserFeedEntry::Table, UserFeedEntry::UserFeedId))),
            )
            .join(
                JoinType::InnerJoin,
                Tag::Table,
                Expr::col((Tag::Table, Tag::Id))
                    .eq(Expr::col((UserFeedTag::Table, UserFeedTag::TagId))),
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

pub fn insert_many_for_all_users(data: &[InsertMany], feed_id: Uuid) -> WithQuery {
    let input = Alias::new("input");

    let input_cte = Query::select()
        .expr(Expr::col(Asterisk))
        .from_values(
            data.iter()
                .map(|e| {
                    if let Some(id) = e.id {
                        (e.feed_entry_id, id).into_value_tuple()
                    } else {
                        (e.feed_entry_id).into_value_tuple()
                    }
                })
                .collect::<Vec<_>>(),
            Alias::new("row"),
        )
        .to_owned();

    let mut cte = CommonTableExpression::new()
        .query(input_cte)
        .column(UserFeedEntry::FeedEntryId)
        .table_name(input.clone())
        .to_owned();

    let mut select_from = Query::select()
        .column((input.clone(), UserFeedEntry::FeedEntryId))
        .columns([
            (UserFeed::Table, UserFeed::Id),
            (UserFeed::Table, UserFeed::UserId),
        ])
        .from(input.clone())
        .inner_join(
            UserFeed::Table,
            Expr::col((UserFeed::Table, UserFeed::FeedId)).eq(feed_id),
        )
        .to_owned();

    let mut columns = vec![
        UserFeedEntry::FeedEntryId,
        UserFeedEntry::UserFeedId,
        UserFeedEntry::UserId,
    ];

    if data.iter().any(|e| e.id.is_some()) {
        cte.column(UserFeedEntry::Id);
        select_from.column((input, UserFeedEntry::Id));
        columns.push(UserFeedEntry::Id);
    }

    let insert = Query::insert()
        .into_table(UserFeedEntry::Table)
        .columns(columns)
        .select_from(select_from)
        .unwrap()
        .on_conflict(
            OnConflict::columns([UserFeedEntry::UserFeedId, UserFeedEntry::FeedEntryId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    insert.with(Query::with().cte(cte.to_owned()).to_owned())
}

pub fn update(id: Uuid, user_id: Uuid, has_read: Option<bool>) -> UpdateStatement {
    let mut query = Query::update()
        .table(UserFeedEntry::Table)
        .value(UserFeedEntry::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col((UserFeedEntry::Table, UserFeedEntry::Id)).eq(id))
        .and_where(Expr::col((UserFeedEntry::Table, UserFeedEntry::UserId)).eq(user_id))
        .to_owned();

    if let Some(has_read) = has_read {
        query.value(UserFeedEntry::HasRead, has_read);
    }

    query
}
