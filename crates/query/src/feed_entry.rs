use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::{
    feed_entry::FeedEntryParams,
    subscription_entry::{
        SubscriptionEntryBooleanField, SubscriptionEntryDateField, SubscriptionEntryFilter,
        SubscriptionEntryParams, SubscriptionEntryTextField,
    },
};
use sea_query::{
    Alias, Asterisk, Expr, Func, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
    SimpleExpr,
};
use uuid::Uuid;

use crate::{
    IntoInsert, IntoSelect,
    filter::{ToColumn, ToSql},
    read_entry::ReadEntry,
    subscription::Subscription,
    subscription_tag::SubscriptionTag,
    tag::Tag,
};

pub enum FeedEntry {
    Table,
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
    FeedId,
}

impl Iden for FeedEntry {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "feed_entries",
                Self::Id => "id",
                Self::Link => "link",
                Self::Title => "title",
                Self::PublishedAt => "published_at",
                Self::Description => "description",
                Self::Author => "author",
                Self::ThumbnailUrl => "thumbnail_url",
                Self::FeedId => "feed_id",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for FeedEntryParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(FeedEntry::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((FeedEntry::Table, FeedEntry::Id)).eq(id));
            })
            .apply_if(self.feed_id, |query, feed_id| {
                query.and_where(Expr::col((FeedEntry::Table, FeedEntry::FeedId)).eq(feed_id));
            })
            .apply_if(self.cursor, |query, (published_at, id)| {
                query.and_where(
                    Expr::tuple([
                        Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)).into(),
                        Expr::col((FeedEntry::Table, FeedEntry::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(published_at).into(),
                        Expr::val(id).into(),
                    ])),
                );
            })
            .order_by((FeedEntry::Table, FeedEntry::PublishedAt), Order::Desc)
            .order_by((FeedEntry::Table, FeedEntry::Id), Order::Desc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct FeedEntryInsert<'a> {
    pub id: Uuid,
    pub link: &'a str,
    pub title: &'a str,
    pub published_at: DateTime<Utc>,
    pub description: Option<&'a str>,
    pub author: Option<&'a str>,
    pub thumbnail_url: Option<&'a str>,
    pub feed_id: Uuid,
}

pub struct FeedEntryInsertBatch<I>(pub I);

impl<'a, I: IntoIterator<Item = FeedEntryInsert<'a>>> IntoInsert for FeedEntryInsertBatch<I> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(FeedEntry::Table)
            .columns([
                FeedEntry::Id,
                FeedEntry::Link,
                FeedEntry::Title,
                FeedEntry::PublishedAt,
                FeedEntry::Description,
                FeedEntry::Author,
                FeedEntry::ThumbnailUrl,
                FeedEntry::FeedId,
            ])
            .on_conflict(
                OnConflict::columns([FeedEntry::FeedId, FeedEntry::Link])
                    .update_columns([
                        FeedEntry::Title,
                        FeedEntry::PublishedAt,
                        FeedEntry::Description,
                        FeedEntry::Author,
                        FeedEntry::ThumbnailUrl,
                    ])
                    .to_owned(),
            )
            .to_owned();

        for entry in self.0 {
            query.values_panic([
                entry.id.into(),
                entry.link.into(),
                entry.title.into(),
                entry.published_at.into(),
                entry.description.into(),
                entry.author.into(),
                entry.thumbnail_url.into(),
                entry.feed_id.into(),
            ]);
        }

        query
    }
}

impl IntoSelect for SubscriptionEntryParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column((FeedEntry::Table, Asterisk))
            .expr_as(
                Expr::col((Subscription::Table, Subscription::Id)),
                Alias::new("subscription_id"),
            )
            .column((Subscription::Table, Subscription::UserId))
            .from(FeedEntry::Table)
            .inner_join(
                Subscription::Table,
                Expr::col((Subscription::Table, Subscription::FeedId))
                    .eq(Expr::col((FeedEntry::Table, FeedEntry::FeedId))),
            )
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Subscription::Table, Subscription::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, (published_at, id)| {
                query.and_where(
                    Expr::tuple([
                        Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)).into(),
                        Expr::col((FeedEntry::Table, FeedEntry::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(published_at).into(),
                        Expr::val(id).into(),
                    ])),
                );
            })
            .order_by((FeedEntry::Table, FeedEntry::PublishedAt), Order::Desc)
            .order_by((FeedEntry::Table, FeedEntry::Id), Order::Desc)
            .to_owned();

        if let Some(filter) = self.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(self.feed_entry_id, |query, feed_entry_id| {
                    query.and_where(Expr::col((FeedEntry::Table, FeedEntry::Id)).eq(feed_entry_id));
                })
                .apply_if(self.subscription_id, |query, subscription_id| {
                    query.and_where(
                        Expr::col((Subscription::Table, Subscription::Id)).eq(subscription_id),
                    );
                })
                .apply_if(self.has_read, |query, has_read| {
                    let mut subquery = Expr::exists(
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
                    );

                    if !has_read {
                        subquery = subquery.not();
                    }

                    query.and_where(subquery);
                })
                .apply_if(self.tags, |query, tags| {
                    query.and_where(Expr::exists(
                        Query::select()
                            .expr(Expr::val("1"))
                            .from(SubscriptionTag::Table)
                            .and_where(
                                Expr::col((
                                    SubscriptionTag::Table,
                                    SubscriptionTag::SubscriptionId,
                                ))
                                .eq(Expr::col((Subscription::Table, Subscription::Id))),
                            )
                            .and_where(
                                Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))
                                    .is_in(tags),
                            )
                            .to_owned(),
                    ));
                });
        }

        if self.with_read_entry {
            query
                .column((ReadEntry::Table, ReadEntry::CreatedAt))
                .expr_as(
                    Expr::col((ReadEntry::Table, ReadEntry::SubscriptionId)).is_not_null(),
                    Alias::new("has_read"),
                )
                .left_join(
                    ReadEntry::Table,
                    Expr::col((ReadEntry::Table, ReadEntry::SubscriptionId))
                        .eq(Expr::col((Subscription::Table, Subscription::Id)))
                        .and(
                            Expr::col((ReadEntry::Table, ReadEntry::FeedEntryId))
                                .eq(Expr::col((FeedEntry::Table, FeedEntry::Id))),
                        ),
                );
        }

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct UnreadCountSelectMany<T> {
    pub subscription_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoSelect for UnreadCountSelectMany<I> {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((Subscription::Table, Subscription::Id))
            .expr_as(
                Func::count(Expr::col((FeedEntry::Table, FeedEntry::Id))),
                Alias::new("unread_count"),
            )
            .from(FeedEntry::Table)
            .inner_join(
                Subscription::Table,
                Expr::col((Subscription::Table, Subscription::FeedId))
                    .eq(Expr::col((FeedEntry::Table, FeedEntry::FeedId))),
            )
            .and_where(
                Expr::col((Subscription::Table, Subscription::Id)).is_in(self.subscription_ids),
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
            .to_owned()
    }
}

impl ToColumn for SubscriptionEntryTextField {
    fn to_column(self) -> Expr {
        match self {
            Self::Link => Expr::col((FeedEntry::Table, FeedEntry::Link)),
            Self::Title => Expr::col((FeedEntry::Table, FeedEntry::Title)),
            Self::Description => Expr::col((FeedEntry::Table, FeedEntry::Description)),
            Self::Author => Expr::col((FeedEntry::Table, FeedEntry::Author)),
            Self::Tag => Expr::col((Tag::Table, Tag::Title)),
        }
    }
}

impl ToColumn for SubscriptionEntryBooleanField {
    fn to_column(self) -> Expr {
        match self {
            Self::HasRead => Expr::col((ReadEntry::Table, ReadEntry::SubscriptionId)),
        }
    }
}

impl ToColumn for SubscriptionEntryDateField {
    fn to_column(self) -> Expr {
        match self {
            Self::PublishedAt => Expr::col((FeedEntry::Table, FeedEntry::PublishedAt)),
        }
    }
}

impl ToSql for SubscriptionEntryFilter {
    fn to_sql(self) -> SimpleExpr {
        match self {
            Self::Text { field, op } => match field {
                SubscriptionEntryTextField::Tag => Expr::exists(
                    Query::select()
                        .expr(Expr::val("1"))
                        .from(SubscriptionTag::Table)
                        .inner_join(
                            Tag::Table,
                            Expr::col((Tag::Table, Tag::Id))
                                .eq(Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))),
                        )
                        .and_where(
                            Expr::col((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                                .eq(Expr::col((Subscription::Table, Subscription::Id))),
                        )
                        .and_where((field.to_column(), op).to_sql())
                        .to_owned(),
                ),
                _ => (field.to_column(), op).to_sql(),
            },
            Self::Boolean { field, op } => (field.to_column(), op).to_sql(),
            Self::Date { field, op } => (field.to_column(), op).to_sql(),
            Self::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = and.and(condition)
                }

                and
            }
            Self::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = or.or(condition)
                }

                or
            }
            Self::Not(filter) => filter.to_sql().not(),
            _ => unreachable!(),
        }
    }
}
