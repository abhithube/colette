use chrono::{DateTime, Utc};
use colette_core::{
    feed_entry::FeedEntryFindParams,
    subscription_entry::{
        SubscriptionEntryBooleanField, SubscriptionEntryDateField, SubscriptionEntryFilter,
        SubscriptionEntryFindByIdParams, SubscriptionEntryFindParams, SubscriptionEntryTextField,
    },
};
use colette_model::{feed_entries, read_entries, subscription_tags, subscriptions, tags};
use sea_query::{
    Alias, Asterisk, Expr, Func, InsertStatement, OnConflict, Order, Query, SelectStatement,
    SimpleExpr,
};
use url::Url;
use uuid::Uuid;

use crate::{
    IntoInsert, IntoSelect,
    filter::{ToColumn, ToSql},
};

impl IntoSelect for FeedEntryFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(feed_entries::Entity)
            .apply_if(self.id, |query, id| {
                query.and_where(
                    Expr::col((feed_entries::Entity, feed_entries::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(self.feed_id, |query, feed_id| {
                query.and_where(
                    Expr::col((feed_entries::Entity, feed_entries::Column::FeedId))
                        .eq(feed_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.timestamp()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                );
            })
            .order_by(
                (feed_entries::Entity, feed_entries::Column::PublishedAt),
                Order::Desc,
            )
            .order_by(
                (feed_entries::Entity, feed_entries::Column::Id),
                Order::Desc,
            )
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for SubscriptionEntryFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .columns([
                (feed_entries::Entity, feed_entries::Column::Id),
                (feed_entries::Entity, feed_entries::Column::Link),
                (feed_entries::Entity, feed_entries::Column::Title),
                (feed_entries::Entity, feed_entries::Column::PublishedAt),
                (feed_entries::Entity, feed_entries::Column::Description),
                (feed_entries::Entity, feed_entries::Column::Author),
                (feed_entries::Entity, feed_entries::Column::ThumbnailUrl),
                (feed_entries::Entity, feed_entries::Column::FeedId),
            ])
            .expr_as(
                Expr::col((subscriptions::Entity, subscriptions::Column::Id)),
                Alias::new("subscription_id"),
            )
            .column((subscriptions::Entity, subscriptions::Column::UserId))
            .expr_as(
                Expr::col((read_entries::Entity, read_entries::Column::SubscriptionId))
                    .is_not_null(),
                Alias::new("has_read"),
            )
            .from(feed_entries::Entity)
            .inner_join(
                subscriptions::Entity,
                Expr::col((subscriptions::Entity, subscriptions::Column::FeedId)).eq(Expr::col((
                    feed_entries::Entity,
                    feed_entries::Column::FeedId,
                ))),
            )
            .left_join(
                read_entries::Entity,
                Expr::col((read_entries::Entity, read_entries::Column::SubscriptionId))
                    .eq(Expr::col((
                        subscriptions::Entity,
                        subscriptions::Column::Id,
                    )))
                    .and(
                        Expr::col((read_entries::Entity, read_entries::Column::FeedEntryId))
                            .eq(Expr::col((feed_entries::Entity, feed_entries::Column::Id))),
                    ),
            )
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((subscriptions::Entity, subscriptions::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.timestamp()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                );
            })
            .order_by(
                (feed_entries::Entity, feed_entries::Column::PublishedAt),
                Order::Desc,
            )
            .order_by(
                (feed_entries::Entity, feed_entries::Column::Id),
                Order::Desc,
            )
            .to_owned();

        if let Some(filter) = self.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(self.id, |query, id| {
                    query.and_where(
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id))
                            .eq(id.to_string()),
                    );
                })
                .apply_if(self.has_read, |query, has_read| {
                    let mut subquery = Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(read_entries::Entity)
                            .and_where(
                                Expr::col((
                                    read_entries::Entity,
                                    read_entries::Column::FeedEntryId,
                                ))
                                .eq(Expr::col((feed_entries::Entity, feed_entries::Column::Id))),
                            )
                            .and_where(
                                Expr::col((
                                    read_entries::Entity,
                                    read_entries::Column::SubscriptionId,
                                ))
                                .eq(Expr::col((
                                    subscriptions::Entity,
                                    subscriptions::Column::Id,
                                ))),
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
                            .expr(Expr::val(1))
                            .from(subscription_tags::Entity)
                            .and_where(
                                Expr::col((
                                    subscription_tags::Entity,
                                    subscription_tags::Column::SubscriptionId,
                                ))
                                .eq(Expr::col((
                                    subscriptions::Entity,
                                    subscriptions::Column::Id,
                                ))),
                            )
                            .and_where(
                                Expr::col((
                                    subscription_tags::Entity,
                                    subscription_tags::Column::TagId,
                                ))
                                .is_in(tags.into_iter().map(String::from)),
                            )
                            .to_owned(),
                    ));
                });
        }

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for SubscriptionEntryFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((feed_entries::Entity, feed_entries::Column::Id))
            .column((subscriptions::Entity, subscriptions::Column::UserId))
            .from(feed_entries::Entity)
            .inner_join(
                subscriptions::Entity,
                Expr::col((subscriptions::Entity, subscriptions::Column::FeedId)).eq(Expr::col((
                    feed_entries::Entity,
                    feed_entries::Column::FeedId,
                ))),
            )
            .and_where(
                Expr::col((feed_entries::Entity, feed_entries::Column::Id))
                    .eq(self.feed_entry_id.to_string()),
            )
            .to_owned()
    }
}

pub struct FeedEntryUpsert {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub feed_id: Uuid,
}

impl IntoInsert for Vec<FeedEntryUpsert> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(feed_entries::Entity)
            .columns([
                feed_entries::Column::Id,
                feed_entries::Column::Link,
                feed_entries::Column::Title,
                feed_entries::Column::PublishedAt,
                feed_entries::Column::Description,
                feed_entries::Column::Author,
                feed_entries::Column::ThumbnailUrl,
                feed_entries::Column::FeedId,
            ])
            .on_conflict(
                OnConflict::columns([feed_entries::Column::FeedId, feed_entries::Column::Link])
                    .update_columns([
                        feed_entries::Column::Title,
                        feed_entries::Column::PublishedAt,
                        feed_entries::Column::Description,
                        feed_entries::Column::Author,
                        feed_entries::Column::ThumbnailUrl,
                    ])
                    .to_owned(),
            )
            .to_owned();

        for entry in self {
            query.values_panic([
                entry.id.to_string().into(),
                entry.link.to_string().into(),
                entry.title.into(),
                entry.published_at.timestamp().into(),
                entry.description.into(),
                entry.author.into(),
                entry.thumbnail_url.map(String::from).into(),
                entry.feed_id.to_string().into(),
            ]);
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
            .column((subscriptions::Entity, subscriptions::Column::Id))
            .expr(Func::count(Expr::col((
                feed_entries::Entity,
                feed_entries::Column::Id,
            ))))
            .from(feed_entries::Entity)
            .inner_join(
                subscriptions::Entity,
                Expr::col((subscriptions::Entity, subscriptions::Column::FeedId)).eq(Expr::col((
                    feed_entries::Entity,
                    feed_entries::Column::FeedId,
                ))),
            )
            .and_where(
                Expr::col((subscriptions::Entity, subscriptions::Column::Id))
                    .is_in(self.subscription_ids),
            )
            .and_where(
                Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(read_entries::Entity)
                        .and_where(
                            Expr::col((read_entries::Entity, read_entries::Column::FeedEntryId))
                                .eq(Expr::col((feed_entries::Entity, feed_entries::Column::Id))),
                        )
                        .and_where(
                            Expr::col((read_entries::Entity, read_entries::Column::SubscriptionId))
                                .eq(Expr::col((
                                    subscriptions::Entity,
                                    subscriptions::Column::Id,
                                ))),
                        )
                        .to_owned(),
                )
                .not(),
            )
            .group_by_col((subscriptions::Entity, subscriptions::Column::Id))
            .to_owned()
    }
}

impl ToColumn for SubscriptionEntryTextField {
    fn to_column(self) -> Expr {
        match self {
            Self::Link => Expr::col((feed_entries::Entity, feed_entries::Column::Link)),
            Self::Title => Expr::col((feed_entries::Entity, feed_entries::Column::Title)),
            Self::Description => {
                Expr::col((feed_entries::Entity, feed_entries::Column::Description))
            }
            Self::Author => Expr::col((feed_entries::Entity, feed_entries::Column::Author)),
            Self::Tag => Expr::col((tags::Entity, tags::Column::Title)),
        }
    }
}

impl ToColumn for SubscriptionEntryBooleanField {
    fn to_column(self) -> Expr {
        match self {
            Self::HasRead => {
                Expr::col((read_entries::Entity, read_entries::Column::SubscriptionId))
            }
        }
    }
}

impl ToColumn for SubscriptionEntryDateField {
    fn to_column(self) -> Expr {
        match self {
            Self::PublishedAt => {
                Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt))
            }
        }
    }
}

impl ToSql for SubscriptionEntryFilter {
    fn to_sql(self) -> SimpleExpr {
        match self {
            SubscriptionEntryFilter::Text { field, op } => match field {
                SubscriptionEntryTextField::Tag => Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(subscription_tags::Entity)
                        .inner_join(
                            tags::Entity,
                            Expr::col((tags::Entity, tags::Column::Id)).eq(Expr::col((
                                subscription_tags::Entity,
                                subscription_tags::Column::TagId,
                            ))),
                        )
                        .and_where(
                            Expr::col((
                                subscription_tags::Entity,
                                subscription_tags::Column::SubscriptionId,
                            ))
                            .eq(Expr::col((
                                subscriptions::Entity,
                                subscriptions::Column::Id,
                            ))),
                        )
                        .and_where((field.to_column(), op).to_sql())
                        .to_owned(),
                ),
                _ => (field.to_column(), op).to_sql(),
            },
            SubscriptionEntryFilter::Boolean { field, op } => (field.to_column(), op).to_sql(),
            SubscriptionEntryFilter::Date { field, op } => (field.to_column(), op).to_sql(),
            SubscriptionEntryFilter::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = and.and(condition)
                }

                and
            }
            SubscriptionEntryFilter::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = or.or(condition)
                }

                or
            }
            SubscriptionEntryFilter::Not(filter) => filter.to_sql().not(),
            _ => unreachable!(),
        }
    }
}
