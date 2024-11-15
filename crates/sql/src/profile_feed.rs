use std::fmt::Write;

use colette_core::feed::Cursor;
use sea_query::{
    Alias, CommonTableExpression, DeleteStatement, Expr, Func, Iden, InsertStatement, JoinType,
    OnConflict, Order, Query, SelectStatement, SimpleExpr, UpdateStatement, WithClause, WithQuery,
};
use uuid::Uuid;

use crate::{
    feed::Feed, profile_feed_entry::ProfileFeedEntry, profile_feed_tag::ProfileFeedTag, tag::Tag,
};

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

impl Iden for ProfileFeed {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "profile_feeds",
                Self::Id => "id",
                Self::Title => "title",
                Self::Pinned => "pinned",
                Self::ProfileId => "profile_id",
                Self::FeedId => "feed_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
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
            JoinType::InnerJoin,
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
        .order_by_expr(
            Func::coalesce([
                Expr::col((ProfileFeed::Table, ProfileFeed::Title)).into(),
                Expr::col((Feed::Table, Feed::Title)).into(),
            ])
            .into(),
            Order::Asc,
        )
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

pub fn insert(
    id: Option<Uuid>,
    pinned: Option<bool>,
    feed_id: i32,
    profile_id: Uuid,
) -> InsertStatement {
    let mut columns = vec![
        ProfileFeed::Pinned,
        ProfileFeed::FeedId,
        ProfileFeed::ProfileId,
    ];
    let mut values: Vec<SimpleExpr> = vec![
        pinned.unwrap_or_default().into(),
        feed_id.into(),
        profile_id.into(),
    ];

    if let Some(id) = id {
        columns.push(ProfileFeed::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(ProfileFeed::Table)
        .columns(columns)
        .values_panic(values)
        .on_conflict(
            OnConflict::columns([ProfileFeed::ProfileId, ProfileFeed::FeedId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(ProfileFeed::Id)
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
