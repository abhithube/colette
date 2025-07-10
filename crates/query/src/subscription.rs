use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{
    Alias, Asterisk, DeleteStatement, Expr, Func, Iden, InsertStatement, JoinType, OnConflict,
    Order, Query, SelectStatement,
};
use uuid::Uuid;

use crate::{
    Dialect, IntoDelete, IntoInsert, IntoSelect, feed::Feed, feed_entry::FeedEntry,
    read_entry::ReadEntry, subscription_tag::SubscriptionTag, tag::Tag,
};

pub enum Subscription {
    Table,
    Id,
    Title,
    Description,
    UserId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Subscription {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "subscriptions",
                Self::Id => "id",
                Self::Title => "title",
                Self::Description => "description",
                Self::FeedId => "feed_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

#[derive(Default)]
pub struct SubscriptionSelect<'a> {
    pub id: Option<Uuid>,
    pub feeds: Option<Vec<Uuid>>,
    pub tags: Option<Vec<Uuid>>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<(&'a str, Uuid)>,
    pub limit: Option<u64>,
    pub with_feed: bool,
    pub with_unread_count: bool,
    pub with_tags: bool,
    pub dialect: Dialect,
}

impl IntoSelect for SubscriptionSelect<'_> {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column((Subscription::Table, Asterisk))
            .from(Subscription::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Subscription::Table, Subscription::Id)).eq(id));
            })
            .apply_if(self.feeds, |query, feeds| {
                query
                    .and_where(Expr::col((Subscription::Table, Subscription::FeedId)).is_in(feeds));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Subscription::Table, Subscription::UserId)).eq(user_id));
            })
            .apply_if(self.tags, |query, tags| {
                query.and_where(Expr::exists(
                    Query::select()
                        .expr(Expr::val("1"))
                        .from(SubscriptionTag::Table)
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                                .eq(Expr::col((Subscription::Table, Subscription::Id))),
                        )
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId)).is_in(tags),
                        )
                        .to_owned(),
                ));
            })
            .apply_if(self.cursor, |query, (title, id)| {
                query.and_where(
                    Expr::tuple([
                        Expr::col(Subscription::Title).into(),
                        Expr::col(Subscription::Id).into(),
                    ])
                    .gt(Expr::tuple([Expr::value(title), Expr::value(id)])),
                );
            })
            .order_by((Subscription::Table, Subscription::Title), Order::Asc)
            .order_by((Subscription::Table, Subscription::Id), Order::Asc)
            .to_owned();

        if self.with_feed {
            query
                .columns([
                    (Feed::Table, Feed::SourceUrl),
                    (Feed::Table, Feed::Link),
                    (Feed::Table, Feed::Description),
                    (Feed::Table, Feed::RefreshedAt),
                    (Feed::Table, Feed::IsCustom),
                ])
                .expr_as(
                    Expr::col((Feed::Table, Feed::Title)),
                    Alias::new("feed_title"),
                )
                .inner_join(
                    Feed::Table,
                    Expr::col((Feed::Table, Feed::Id))
                        .eq(Expr::col((Subscription::Table, Subscription::FeedId))),
                );
        }

        if self.with_unread_count {
            let uc_agg = Alias::new("uc_agg");
            let unread_count = Alias::new("unread_count");

            query
                .expr_as(
                    Func::coalesce([
                        Expr::col((uc_agg.clone(), unread_count.clone())).into(),
                        Expr::cust("0"),
                    ]),
                    unread_count.clone(),
                )
                .join_subquery(
                    JoinType::LeftJoin,
                    Query::select()
                        .column((Subscription::Table, Subscription::Id))
                        .expr_as(
                            Func::count(Expr::col((FeedEntry::Table, FeedEntry::Id))),
                            unread_count,
                        )
                        .from(FeedEntry::Table)
                        .inner_join(
                            Subscription::Table,
                            Expr::col((Subscription::Table, Subscription::FeedId))
                                .eq(Expr::col((FeedEntry::Table, FeedEntry::FeedId))),
                        )
                        .and_where(
                            Expr::exists(
                                Query::select()
                                    .expr(Expr::val("1"))
                                    .from(ReadEntry::Table)
                                    .and_where(
                                        Expr::col((ReadEntry::Table, ReadEntry::FeedEntryId))
                                            .eq(Expr::col((FeedEntry::Table, FeedEntry::Id))),
                                    )
                                    .and_where(
                                        Expr::col((ReadEntry::Table, ReadEntry::SubscriptionId))
                                            .eq(Expr::col((Subscription::Table, Subscription::Id))),
                                    )
                                    .to_owned(),
                            )
                            .not(),
                        )
                        .group_by_col((Subscription::Table, Subscription::Id))
                        .to_owned(),
                    uc_agg.clone(),
                    Expr::col((uc_agg, Subscription::Id))
                        .eq(Expr::col((Subscription::Table, Subscription::Id))),
                );
        }

        if self.with_tags {
            let tags_agg = Alias::new("tags_agg");
            let tags = Alias::new("tags");
            let t = Alias::new("t");

            let agg_expr = match self.dialect {
                Dialect::Postgres => Expr::cust(
                    "jsonb_agg (jsonb_build_object ('id', t.id, 'title', t.title, 'user_id', t.user_id, 'created_at', t.created_at, 'updated_at', t.updated_at) ORDER BY t.title)",
                ),
                Dialect::Sqlite => Expr::cust(
                    "json_group_array (json_object ('id', hex(t.id), 'title', t.title, 'user_id', hex(t.user_id), 'created_at', t.created_at, 'updated_at', t.updated_at) ORDER BY t.title)",
                ),
            };

            query
                .expr_as(
                    Func::coalesce([
                        Expr::col((tags_agg.clone(), tags.clone())).into(),
                        Expr::cust("'[]'"),
                    ]),
                    tags.clone(),
                )
                .join_subquery(
                    JoinType::LeftJoin,
                    Query::select()
                        .column((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                        .expr_as(agg_expr, tags)
                        .from(SubscriptionTag::Table)
                        .join_as(
                            JoinType::InnerJoin,
                            Tag::Table,
                            t.clone(),
                            Expr::col((t, Tag::Id))
                                .eq(Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))),
                        )
                        .group_by_col((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                        .to_owned(),
                    tags_agg.clone(),
                    Expr::col((tags_agg, SubscriptionTag::SubscriptionId))
                        .eq(Expr::col((Subscription::Table, Subscription::Id))),
                );
        }

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct SubscriptionInsert<I> {
    pub subscriptions: I,
    pub user_id: Uuid,
    pub upsert: bool,
}

pub struct SubscriptionBase<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub description: Option<&'a str>,
    pub feed_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'a, I: IntoIterator<Item = SubscriptionBase<'a>>> IntoInsert for SubscriptionInsert<I> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(Subscription::Table)
            .columns([
                Subscription::Id,
                Subscription::Title,
                Subscription::Description,
                Subscription::FeedId,
                Subscription::UserId,
                Subscription::CreatedAt,
                Subscription::UpdatedAt,
            ])
            .to_owned();

        if self.upsert {
            query
                .on_conflict(
                    OnConflict::columns([Subscription::UserId, Subscription::FeedId])
                        .update_columns([
                            Subscription::Title,
                            Subscription::Description,
                            Subscription::UpdatedAt,
                        ])
                        .to_owned(),
                )
                .returning_col(Subscription::Id);
        } else {
            query.on_conflict(
                OnConflict::column(Subscription::Id)
                    .update_columns([Subscription::Title, Subscription::UpdatedAt])
                    .to_owned(),
            );
        }

        for subscription in self.subscriptions.into_iter() {
            query.values_panic([
                subscription.id.into(),
                subscription.title.into(),
                subscription.description.into(),
                subscription.feed_id.into(),
                self.user_id.into(),
                subscription.created_at.into(),
                subscription.updated_at.into(),
            ]);
        }

        query
    }
}

#[derive(Default)]
pub struct SubscriptionDelete {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

impl IntoDelete for SubscriptionDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Subscription::Table)
            .and_where_option(self.id.map(|e| Expr::col(Subscription::Id).eq(e)))
            .and_where_option(self.user_id.map(|e| Expr::col(Subscription::UserId).eq(e)))
            .to_owned()
    }
}
