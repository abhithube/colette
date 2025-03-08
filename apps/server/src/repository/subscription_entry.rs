use colette_core::{
    SubscriptionEntry,
    common::Transaction,
    subscription_entry::{
        Error, SubscriptionEntryBooleanField, SubscriptionEntryById, SubscriptionEntryDateField,
        SubscriptionEntryFilter, SubscriptionEntryFindByIdParams, SubscriptionEntryFindParams,
        SubscriptionEntryRepository, SubscriptionEntryTextField,
    },
};
use colette_model::{
    SubscriptionEntryRow, feed_entries, read_entries, subscription_tags, subscriptions, tags,
};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult,
    sea_query::{Alias, Expr, Order, Query, SimpleExpr},
};

use super::common::{ToColumn, ToSql};

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionEntryRepository {
    db: DatabaseConnection,
}

impl SqliteSubscriptionEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl SubscriptionEntryRepository for SqliteSubscriptionEntryRepository {
    async fn find_subscription_entries(
        &self,
        params: SubscriptionEntryFindParams,
    ) -> Result<Vec<SubscriptionEntry>, Error> {
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
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((subscriptions::Entity, subscriptions::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(params.cursor, |query, cursor| {
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

        if let Some(filter) = params.filter {
            query.and_where(filter.to_sql());
        } else {
            query
                .apply_if(params.id, |query, id| {
                    query.and_where(
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id))
                            .eq(id.to_string()),
                    );
                })
                .apply_if(params.has_read, |query, has_read| {
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
                .apply_if(params.tags, |query, tags| {
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

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let subscription_entries =
            SubscriptionEntryRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await
                .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(subscription_entries)
    }

    async fn find_subscription_entry_by_id(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryFindByIdParams,
    ) -> Result<SubscriptionEntryById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
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
                    .eq(params.feed_entry_id.to_string()),
            )
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
            .await?
        else {
            return Err(Error::NotFound(params.feed_entry_id));
        };

        Ok(SubscriptionEntryById {
            feed_entry_id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
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
