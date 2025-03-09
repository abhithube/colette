use colette_core::subscription::{
    SubscriptionCreateParams, SubscriptionDeleteParams, SubscriptionFindByIdParams,
    SubscriptionFindParams, SubscriptionUpdateParams,
};
use colette_model::{feeds, subscription_tags, subscriptions};
use sea_query::{
    Alias, DeleteStatement, Expr, InsertStatement, OnConflict, Order, Query, SelectStatement,
    UpdateStatement,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

impl IntoSelect for SubscriptionFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .columns([
                (subscriptions::Entity, subscriptions::Column::Id),
                (subscriptions::Entity, subscriptions::Column::Title),
                (subscriptions::Entity, subscriptions::Column::UserId),
                (subscriptions::Entity, subscriptions::Column::CreatedAt),
                (subscriptions::Entity, subscriptions::Column::UpdatedAt),
            ])
            .columns([
                (feeds::Entity, feeds::Column::Link),
                (feeds::Entity, feeds::Column::XmlUrl),
                (feeds::Entity, feeds::Column::Description),
                (feeds::Entity, feeds::Column::RefreshedAt),
            ])
            .expr_as(
                Expr::col((feeds::Entity, feeds::Column::Id)),
                Alias::new("feed_id"),
            )
            .expr_as(
                Expr::col((feeds::Entity, feeds::Column::Title)),
                Alias::new("feed_title"),
            )
            .from(subscriptions::Entity)
            .inner_join(
                feeds::Entity,
                Expr::col((feeds::Entity, feeds::Column::Id)).eq(Expr::col((
                    subscriptions::Entity,
                    subscriptions::Column::FeedId,
                ))),
            )
            .apply_if(self.id, |query, id| {
                query.and_where(
                    Expr::col((subscriptions::Entity, subscriptions::Column::Id))
                        .eq(id.to_string()),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((subscriptions::Entity, subscriptions::Column::UserId))
                        .eq(user_id.to_string()),
                );
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
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::tuple([
                        Expr::col(subscriptions::Column::Title).into(),
                        Expr::col(subscriptions::Column::Id).into(),
                    ])
                    .gt(Expr::tuple([
                        Expr::value(cursor.title),
                        Expr::value(cursor.id.to_string()),
                    ])),
                );
            })
            .order_by(
                (subscriptions::Entity, subscriptions::Column::Title),
                Order::Asc,
            )
            .order_by(
                (subscriptions::Entity, subscriptions::Column::Id),
                Order::Asc,
            )
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for SubscriptionFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((subscriptions::Entity, subscriptions::Column::Id))
            .column((subscriptions::Entity, subscriptions::Column::UserId))
            .from(subscriptions::Entity)
            .and_where(
                Expr::col((subscriptions::Entity, subscriptions::Column::Id))
                    .eq(self.id.to_string()),
            )
            .to_owned()
    }
}

impl IntoInsert for SubscriptionCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(subscriptions::Entity)
            .columns([
                subscriptions::Column::Id,
                subscriptions::Column::Title,
                subscriptions::Column::FeedId,
                subscriptions::Column::UserId,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.title.clone().into(),
                self.feed_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for SubscriptionUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(subscriptions::Entity)
            .and_where(Expr::col(subscriptions::Column::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(subscriptions::Column::Title, title);
        }

        query
    }
}

impl IntoDelete for SubscriptionDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(subscriptions::Entity)
            .and_where(Expr::col(subscriptions::Column::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

pub struct SubscriptionUpsert {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub user_id: Uuid,
}

impl IntoInsert for SubscriptionUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(subscriptions::Entity)
            .columns([
                subscriptions::Column::Id,
                subscriptions::Column::Title,
                subscriptions::Column::FeedId,
                subscriptions::Column::UserId,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.title.into(),
                self.feed_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([subscriptions::Column::UserId, subscriptions::Column::FeedId])
                    .update_column(subscriptions::Column::Title)
                    .to_owned(),
            )
            .returning_col(subscriptions::Column::Id)
            .to_owned()
    }
}
