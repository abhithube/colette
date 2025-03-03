use colette_core::{
    FeedEntry,
    common::IdParams,
    feed_entry::{
        Error, FeedEntryBooleanField, FeedEntryById, FeedEntryDateField, FeedEntryFilter,
        FeedEntryFindParams, FeedEntryRepository, FeedEntryTextField, FeedEntryUpdateData,
    },
};
use colette_model::{UfeWithFe, feed_entries, tags, user_feed_entries, user_feed_tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, TransactionTrait,
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
        let mut query = user_feed_entries::Entity::find()
            .find_also_related(feed_entries::Entity)
            .apply_if(params.user_id, |query, user_id| {
                query.filter(user_feed_entries::Column::UserId.eq(user_id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((user_feed_entries::Entity, user_feed_entries::Column::Id))
                            .into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.to_rfc3339()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                )
            })
            .order_by_desc(feed_entries::Column::PublishedAt)
            .order_by_desc(user_feed_entries::Column::Id)
            .limit(params.limit.map(|e| e as u64));

        if let Some(filter) = params.filter {
            query = query.filter(filter.to_sql());
        } else {
            query = query
                .apply_if(params.id, |query, id| {
                    query.filter(user_feed_entries::Column::Id.eq(id.to_string()))
                })
                .apply_if(params.has_read, |query, has_read| {
                    query.filter(user_feed_entries::Column::HasRead.eq(has_read))
                })
                .apply_if(params.tags, |query, tags| {
                    query.filter(Expr::exists(
                        Query::select()
                            .expr(Expr::val(1))
                            .from(user_feed_tags::Entity)
                            .and_where(
                                Expr::col(user_feed_tags::Column::UserFeedId)
                                    .eq(Expr::col(user_feed_entries::Column::UserFeedId)),
                            )
                            .and_where(
                                user_feed_tags::Column::TagId
                                    .is_in(tags.into_iter().map(String::from).collect::<Vec<_>>()),
                            )
                            .to_owned(),
                    ))
                });
        }

        let feed_entries = query.all(&self.db).await.map(|e| {
            e.into_iter()
                .filter_map(|(ufe, fe)| fe.map(|fe| UfeWithFe { ufe, fe }.into()))
                .collect()
        })?;

        Ok(feed_entries)
    }

    async fn find_feed_entry_by_id(&self, id: Uuid) -> Result<FeedEntryById, Error> {
        let Some((id, user_id)) = user_feed_entries::Entity::find()
            .select_only()
            .columns([
                user_feed_entries::Column::Id,
                user_feed_entries::Column::UserId,
            ])
            .filter(user_feed_entries::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(&self.db)
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
        params: IdParams,
        data: FeedEntryUpdateData,
    ) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(feed_entry) = user_feed_entries::Entity::find_by_id(params.id)
            .one(&tx)
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };
        if feed_entry.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut feed_entry = feed_entry.into_active_model();

        if let Some(has_read) = data.has_read {
            feed_entry.has_read = ActiveValue::Set(has_read.into());
        }

        if feed_entry.is_changed() {
            feed_entry.update(&tx).await?;
        }

        tx.commit().await?;

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
                user_feed_entries::Entity,
                user_feed_entries::Column::HasRead,
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
                user_feed_entries::Entity,
                user_feed_entries::Column::CreatedAt,
            )),
            Self::UpdatedAt => Expr::col((
                user_feed_entries::Entity,
                user_feed_entries::Column::UpdatedAt,
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
                        .from(user_feed_tags::Entity)
                        .inner_join(
                            tags::Entity,
                            Expr::col((tags::Entity, tags::Column::Id)).eq(Expr::col((
                                user_feed_tags::Entity,
                                user_feed_tags::Column::TagId,
                            ))),
                        )
                        .and_where(
                            Expr::col((user_feed_tags::Entity, user_feed_tags::Column::UserFeedId))
                                .eq(Expr::col((
                                    user_feed_entries::Entity,
                                    user_feed_entries::Column::UserFeedId,
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
