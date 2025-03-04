use colette_core::{
    FeedEntry,
    common::Transaction,
    feed_entry::{
        Error, FeedEntryBooleanField, FeedEntryById, FeedEntryDateField, FeedEntryFilter,
        FeedEntryFindParams, FeedEntryRepository, FeedEntryTextField, FeedEntryUpdateData,
    },
};
use colette_model::{
    SubscriptionEntryWithFe, feed_entries, subscription_entries, subscription_tags, tags,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    prelude::Expr,
    sea_query::{Query, SimpleExpr},
};
use uuid::Uuid;

use super::common::{ToColumn, ToSql};

#[derive(Debug, Clone)]
pub struct SqliteFeedEntryRepository {
    db: DatabaseConnection,
}

impl SqliteFeedEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for SqliteFeedEntryRepository {
    async fn find_feed_entries(
        &self,
        params: FeedEntryFindParams,
    ) -> Result<Vec<FeedEntry>, Error> {
        let mut query = subscription_entries::Entity::find()
            .find_also_related(feed_entries::Entity)
            .apply_if(params.user_id, |query, user_id| {
                query.filter(subscription_entries::Column::UserId.eq(user_id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((
                            subscription_entries::Entity,
                            subscription_entries::Column::Id,
                        ))
                        .into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.to_rfc3339()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                )
            })
            .order_by_desc(feed_entries::Column::PublishedAt)
            .order_by_desc(subscription_entries::Column::Id)
            .limit(params.limit.map(|e| e as u64));

        if let Some(filter) = params.filter {
            query = query.filter(filter.to_sql());
        } else {
            query = query
                .apply_if(params.id, |query, id| {
                    query.filter(subscription_entries::Column::Id.eq(id.to_string()))
                })
                .apply_if(params.has_read, |query, has_read| {
                    query.filter(subscription_entries::Column::HasRead.eq(has_read))
                })
                .apply_if(params.tags, |query, tags| {
                    query.filter(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(subscription_tags::Entity)
                            .and_where(
                                Expr::col(subscription_tags::Column::SubscriptionId)
                                    .eq(Expr::col(subscription_entries::Column::SubscriptionId)),
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
                .filter_map(|(se, fe)| fe.map(|fe| SubscriptionEntryWithFe { se, fe }.into()))
                .collect()
        })?;

        Ok(feed_entries)
    }

    async fn find_feed_entry_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<FeedEntryById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((id, user_id)) = subscription_entries::Entity::find()
            .select_only()
            .columns([
                subscription_entries::Column::Id,
                subscription_entries::Column::UserId,
            ])
            .filter(subscription_entries::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(FeedEntryById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
    }

    async fn update_feed_entry(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: FeedEntryUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let mut model = subscription_entries::ActiveModel {
            id: ActiveValue::Unchanged(id.to_string()),
            ..Default::default()
        };

        if let Some(has_read) = data.has_read {
            model.has_read = ActiveValue::Set(has_read.into());
        }

        if model.is_changed() {
            model.update(tx).await?;
        }

        Ok(())
    }
}

impl ToColumn for FeedEntryTextField {
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

impl ToColumn for FeedEntryBooleanField {
    fn to_column(self) -> Expr {
        match self {
            Self::HasRead => Expr::col((
                subscription_entries::Entity,
                subscription_entries::Column::HasRead,
            )),
        }
    }
}

impl ToColumn for FeedEntryDateField {
    fn to_column(self) -> Expr {
        match self {
            Self::PublishedAt => {
                Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt))
            }
            Self::CreatedAt => Expr::col((
                subscription_entries::Entity,
                subscription_entries::Column::CreatedAt,
            )),
            Self::UpdatedAt => Expr::col((
                subscription_entries::Entity,
                subscription_entries::Column::UpdatedAt,
            )),
        }
    }
}

impl ToSql for FeedEntryFilter {
    fn to_sql(self) -> SimpleExpr {
        match self {
            FeedEntryFilter::Text { field, op } => match field {
                FeedEntryTextField::Tag => Expr::exists(
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
                                subscription_entries::Entity,
                                subscription_entries::Column::SubscriptionId,
                            ))),
                        )
                        .and_where((field.to_column(), op).to_sql())
                        .to_owned(),
                ),
                _ => (field.to_column(), op).to_sql(),
            },
            FeedEntryFilter::Boolean { field, op } => (field.to_column(), op).to_sql(),
            FeedEntryFilter::Date { field, op } => (field.to_column(), op).to_sql(),
            FeedEntryFilter::And(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut and = conditions.swap_remove(0);

                for condition in conditions {
                    and = and.and(condition)
                }

                and
            }
            FeedEntryFilter::Or(filters) => {
                let mut conditions = filters.into_iter().map(|e| e.to_sql()).collect::<Vec<_>>();
                let mut or = conditions.swap_remove(0);

                for condition in conditions {
                    or = or.or(condition)
                }

                or
            }
            FeedEntryFilter::Not(filter) => filter.to_sql().not(),
            _ => unreachable!(),
        }
    }
}
