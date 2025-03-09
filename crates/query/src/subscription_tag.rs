use std::fmt::Write;

use colette_core::tag::TagById;
use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
    SimpleExpr,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect, tag::Tag};

pub enum SubscriptionTag {
    Table,
    SubscriptionId,
    TagId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for SubscriptionTag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "subscription_tags",
                Self::SubscriptionId => "subscription_id",
                Self::TagId => "tag_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct SubscriptionTagSelectMany<T> {
    pub subscription_ids: T,
}

impl<V: Into<SimpleExpr>, I: IntoIterator<Item = V>> IntoSelect for SubscriptionTagSelectMany<I> {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
            .columns([
                (Tag::Table, Tag::Id),
                (Tag::Table, Tag::Title),
                (Tag::Table, Tag::CreatedAt),
                (Tag::Table, Tag::UpdatedAt),
                (Tag::Table, Tag::UserId),
            ])
            .from(SubscriptionTag::Table)
            .inner_join(
                Tag::Table,
                Expr::col((Tag::Table, Tag::Id))
                    .eq(Expr::col((SubscriptionTag::Table, SubscriptionTag::TagId))),
            )
            .and_where(
                Expr::col((SubscriptionTag::Table, SubscriptionTag::SubscriptionId))
                    .is_in(self.subscription_ids),
            )
            .order_by((Tag::Table, Tag::Title), Order::Asc)
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
            .into_table(SubscriptionTag::Table)
            .columns([
                SubscriptionTag::SubscriptionId,
                SubscriptionTag::TagId,
                SubscriptionTag::UserId,
            ])
            .values_panic([
                self.subscription_id.to_string().into(),
                self.tag_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([SubscriptionTag::SubscriptionId, SubscriptionTag::TagId])
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
            .from_table(SubscriptionTag::Table)
            .and_where(
                Expr::col(SubscriptionTag::SubscriptionId).eq(self.subscription_id.to_string()),
            )
            .and_where(Expr::col(SubscriptionTag::TagId).is_not_in(self.tag_ids))
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
            .into_table(SubscriptionTag::Table)
            .columns([
                SubscriptionTag::SubscriptionId,
                SubscriptionTag::TagId,
                SubscriptionTag::UserId,
            ])
            .on_conflict(
                OnConflict::columns([SubscriptionTag::SubscriptionId, SubscriptionTag::TagId])
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
