use std::fmt::Write;

use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query, SelectStatement,
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

pub struct SubscriptionTagSelect<T> {
    pub subscription_ids: T,
}

impl<I: IntoIterator<Item = Uuid>> IntoSelect for SubscriptionTagSelect<I> {
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

pub struct SubscriptionTagDelete<I> {
    pub subscription_id: Uuid,
    pub tag_ids: I,
}

impl<I: IntoIterator<Item = Uuid>> IntoDelete for SubscriptionTagDelete<I> {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(SubscriptionTag::Table)
            .and_where(Expr::col(SubscriptionTag::SubscriptionId).eq(self.subscription_id))
            .and_where(Expr::col(SubscriptionTag::TagId).is_not_in(self.tag_ids))
            .to_owned()
    }
}

pub struct SubscriptionTagById {
    pub id: Uuid,
    pub user_id: Uuid,
}

pub struct SubscriptionTagInsert<I> {
    pub subscription_id: Uuid,
    pub tags: I,
}

impl<I: IntoIterator<Item = SubscriptionTagById>> IntoInsert for SubscriptionTagInsert<I> {
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
                self.subscription_id.into(),
                tag.id.into(),
                tag.user_id.into(),
            ]);
        }

        query
    }
}
