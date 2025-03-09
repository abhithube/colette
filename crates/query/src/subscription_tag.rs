use colette_core::tag::TagById;
use colette_model::{subscription_tags, tags};
use sea_query::{
    DeleteStatement, Expr, InsertStatement, OnConflict, Order, Query, SelectStatement, SimpleExpr,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect};

pub struct SubscriptionTagSelectMany<T> {
    pub subscription_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoSelect for SubscriptionTagSelectMany<I> {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((
                subscription_tags::Entity,
                subscription_tags::Column::SubscriptionId,
            ))
            .columns([
                (tags::Entity, tags::Column::Id),
                (tags::Entity, tags::Column::Title),
                (tags::Entity, tags::Column::CreatedAt),
                (tags::Entity, tags::Column::UpdatedAt),
                (tags::Entity, tags::Column::UserId),
            ])
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
                .is_in(self.subscription_ids),
            )
            .order_by((tags::Entity, tags::Column::Title), Order::Asc)
            .to_owned()
    }
}

pub struct SubscriptionTagUpsert {
    pub subscription_id: Uuid,
    pub tag_id: Uuid,
    pub user_id: Uuid,
}

impl IntoInsert for SubscriptionTagUpsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(subscription_tags::Entity)
            .columns([
                subscription_tags::Column::SubscriptionId,
                subscription_tags::Column::TagId,
                subscription_tags::Column::UserId,
            ])
            .values_panic([
                self.subscription_id.to_string().into(),
                self.tag_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([
                    subscription_tags::Column::SubscriptionId,
                    subscription_tags::Column::TagId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned()
    }
}

pub struct SubscriptionTagDeleteMany<T> {
    pub subscription_id: Uuid,
    pub tag_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoDelete for SubscriptionTagDeleteMany<I> {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(subscription_tags::Entity)
            .and_where(
                Expr::col(subscription_tags::Column::SubscriptionId)
                    .eq(self.subscription_id.to_string()),
            )
            .and_where(Expr::col(subscription_tags::Column::TagId).is_not_in(self.tag_ids))
            .to_owned()
    }
}

pub struct SubscriptionTagUpsertMany {
    pub subscription_id: Uuid,
    pub tags: Vec<TagById>,
}

impl IntoInsert for SubscriptionTagUpsertMany {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .into_table(subscription_tags::Entity)
            .columns([
                subscription_tags::Column::SubscriptionId,
                subscription_tags::Column::TagId,
                subscription_tags::Column::UserId,
            ])
            .on_conflict(
                OnConflict::columns([
                    subscription_tags::Column::SubscriptionId,
                    subscription_tags::Column::TagId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned();

        for tag in self.tags {
            query.values_panic([
                self.subscription_id.to_string().into(),
                tag.id.to_string().into(),
                tag.user_id.to_string().into(),
            ]);
        }

        query
    }
}
