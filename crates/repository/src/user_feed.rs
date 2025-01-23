use std::fmt::Write;

use colette_core::feed::Cursor;
use sea_query::{
    Alias, CommonTableExpression, DeleteStatement, Expr, Func, Iden, InsertStatement, JoinType,
    OnConflict, Order, Query, SelectStatement, SimpleExpr, UpdateStatement, WithQuery,
};
use uuid::Uuid;

use crate::{feed::Feed, tag::Tag, user_feed_entry::UserFeedEntry, user_feed_tag::UserFeedTag};

#[allow(dead_code)]
pub enum UserFeed {
    Table,
    Id,
    Title,
    FolderId,
    UserId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for UserFeed {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "user_feeds",
                Self::Id => "id",
                Self::Title => "title",
                Self::FolderId => "folder_id",
                Self::UserId => "user_id",
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
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<u64>,
    jsonb_agg: SimpleExpr,
    tags_subquery: Option<SimpleExpr>,
) -> WithQuery {
    let unread_count = Alias::new("unread_count");
    let pf_id = Alias::new("pf_id");

    let unread_count_cte = Query::select()
        .expr_as(Expr::col((UserFeed::Table, UserFeed::Id)), pf_id.clone())
        .expr_as(
            Expr::col((UserFeedEntry::Table, UserFeedEntry::Id)).count(),
            unread_count.clone(),
        )
        .from(UserFeed::Table)
        .join(
            JoinType::InnerJoin,
            UserFeedEntry::Table,
            Expr::col((UserFeedEntry::Table, UserFeedEntry::UserFeedId))
                .eq(Expr::col((UserFeed::Table, UserFeed::Id))),
        )
        .group_by_col((UserFeed::Table, UserFeed::Id))
        .to_owned();

    let tags = Alias::new("tags");

    let json_tags_cte = Query::select()
        .expr_as(Expr::col((UserFeed::Table, UserFeed::Id)), pf_id.clone())
        .expr_as(jsonb_agg, tags.clone())
        .from(UserFeed::Table)
        .join(
            JoinType::InnerJoin,
            UserFeedTag::Table,
            Expr::col((UserFeedTag::Table, UserFeedTag::UserFeedId))
                .eq(Expr::col((UserFeed::Table, UserFeed::Id))),
        )
        .join(
            JoinType::InnerJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::Id))
                .eq(Expr::col((UserFeedTag::Table, UserFeedTag::TagId))),
        )
        .group_by_col((UserFeed::Table, UserFeed::Id))
        .to_owned();

    let json_tags = Alias::new("json_tags");
    let unread_counts = Alias::new("unread_counts");

    let mut select = Query::select()
        .columns([
            (UserFeed::Table, UserFeed::Id),
            (UserFeed::Table, UserFeed::Title),
            (UserFeed::Table, UserFeed::FolderId),
        ])
        .columns([(Feed::Table, Feed::Link), (Feed::Table, Feed::XmlUrl)])
        .expr_as(
            Expr::col((Feed::Table, Feed::Title)),
            Alias::new("original_title"),
        )
        .column((json_tags.clone(), tags))
        .expr_as(
            Func::coalesce([
                Expr::col((unread_counts.clone(), unread_count.clone())).into(),
                Expr::val(0_i64).into(),
            ]),
            unread_count,
        )
        .from(UserFeed::Table)
        .join(
            JoinType::InnerJoin,
            Feed::Table,
            Expr::col((Feed::Table, Feed::Id)).eq(Expr::col((UserFeed::Table, UserFeed::FeedId))),
        )
        .join(
            JoinType::LeftJoin,
            json_tags.clone(),
            Expr::col((json_tags.clone(), pf_id.clone()))
                .eq(Expr::col((UserFeed::Table, UserFeed::Id))),
        )
        .join(
            JoinType::LeftJoin,
            unread_counts.clone(),
            Expr::col((unread_counts.clone(), pf_id))
                .eq(Expr::col((UserFeed::Table, UserFeed::Id))),
        )
        .and_where(Expr::col((UserFeed::Table, UserFeed::UserId)).eq(user_id))
        .and_where_option(id.map(|e| Expr::col((UserFeed::Table, UserFeed::Id)).eq(e)))
        .and_where_option(folder_id.map(|e| Expr::col((UserFeed::Table, UserFeed::FolderId)).eq(e)))
        .and_where_option(tags_subquery)
        .and_where_option(cursor.map(|e| {
            Expr::tuple([
                Func::coalesce([
                    Expr::col((UserFeed::Table, UserFeed::Title)).into(),
                    Expr::col((Feed::Table, Feed::Title)).into(),
                ])
                .into(),
                Expr::col((UserFeed::Table, UserFeed::Id)).into(),
            ])
            .lt(Expr::tuple([
                Expr::val(e.title).into(),
                Expr::val(e.id).into(),
            ]))
        }))
        .order_by_expr(
            Func::coalesce([
                Expr::col((UserFeed::Table, UserFeed::Title)).into(),
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
        Query::with()
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

pub fn select_by_unique_index(user_id: Uuid, feed_id: Uuid) -> SelectStatement {
    Query::select()
        .column(UserFeed::Id)
        .from(UserFeed::Table)
        .and_where(Expr::col(UserFeed::UserId).eq(user_id))
        .and_where(Expr::col(UserFeed::FeedId).eq(feed_id))
        .to_owned()
}

pub fn insert(
    id: Option<Uuid>,
    title: Option<String>,
    folder_id: Option<Uuid>,
    feed_id: Uuid,
    user_id: Uuid,
) -> InsertStatement {
    let mut columns = vec![
        UserFeed::Title,
        UserFeed::FolderId,
        UserFeed::FeedId,
        UserFeed::UserId,
    ];
    let mut values: Vec<SimpleExpr> = vec![
        title.into(),
        folder_id.into(),
        feed_id.into(),
        user_id.into(),
    ];

    if let Some(id) = id {
        columns.push(UserFeed::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(UserFeed::Table)
        .columns(columns)
        .values_panic(values)
        .on_conflict(
            OnConflict::columns([UserFeed::UserId, UserFeed::FeedId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(UserFeed::Id)
        .to_owned()
}

pub fn update(
    id: Uuid,
    user_id: Uuid,
    title: Option<Option<String>>,
    folder_id: Option<Option<Uuid>>,
) -> UpdateStatement {
    let mut query = Query::update()
        .table(UserFeed::Table)
        .value(UserFeed::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col((UserFeed::Table, UserFeed::Id)).eq(id))
        .and_where(Expr::col((UserFeed::Table, UserFeed::UserId)).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(UserFeed::Title, title);
    }
    if let Some(folder_id) = folder_id {
        query.value(UserFeed::FolderId, folder_id);
    }

    query
}

pub fn delete(id: Uuid, user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(UserFeed::Table)
        .and_where(Expr::col((UserFeed::Table, UserFeed::Id)).eq(id))
        .and_where(Expr::col((UserFeed::Table, UserFeed::UserId)).eq(user_id))
        .to_owned()
}
