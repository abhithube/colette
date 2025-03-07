use colette_core::{
    SubscriptionEntry,
    common::Transaction,
    subscription_entry::{
        Error, SubscriptionEntryBooleanField, SubscriptionEntryById, SubscriptionEntryDateField,
        SubscriptionEntryFilter, SubscriptionEntryFindParams, SubscriptionEntryRepository,
        SubscriptionEntryTextField,
    },
};
use colette_model::{
    FeedEntryWithRead, feed_entries, read_entries, subscription_tags, subscriptions, tags,
};
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait,
    prelude::Expr,
    sea_query::{Query, SimpleExpr},
};
use uuid::Uuid;

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
        let mut query = feed_entries::Entity::find()
            .find_also_related(subscriptions::Entity)
            .and_also_related(read_entries::Entity)
            .apply_if(params.user_id, |query, user_id| {
                query.filter(subscriptions::Column::UserId.eq(user_id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.timestamp()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                )
            })
            .order_by_desc(feed_entries::Column::PublishedAt)
            .order_by_desc(subscriptions::Column::Id)
            .limit(params.limit.map(|e| e as u64));

        if let Some(filter) = params.filter {
            query = query.filter(filter.to_sql());
        } else {
            query = query
                .apply_if(params.id, |query, id| {
                    query.filter(feed_entries::Column::Id.eq(id.to_string()))
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

                    query.filter(subquery)
                })
                .apply_if(params.tags, |query, tags| {
                    query.filter(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(subscription_tags::Entity)
                            .and_where(
                                Expr::col(subscription_tags::Column::SubscriptionId)
                                    .eq(Expr::col(subscriptions::Column::Id)),
                            )
                            .and_where(
                                subscription_tags::Column::TagId
                                    .is_in(tags.into_iter().map(String::from).collect::<Vec<_>>()),
                            )
                            .to_owned(),
                    ))
                });
        }

        let feed_entries = query.all(&self.db).await.map(|e| {
            e.into_iter()
                .filter_map(|(fe, subscription, re)| {
                    subscription.map(|subscription| {
                        FeedEntryWithRead {
                            fe,
                            subscription,
                            re,
                        }
                        .into()
                    })
                })
                .collect()
        })?;

        Ok(feed_entries)
    }

    async fn find_subscription_entry_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<SubscriptionEntryById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((feed_entry_id, user_id)) = feed_entries::Entity::find()
            .select_only()
            .column(feed_entries::Column::Id)
            .column(subscriptions::Column::UserId)
            .inner_join(subscriptions::Entity)
            .filter(feed_entries::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(SubscriptionEntryById {
            feed_entry_id: feed_entry_id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
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
