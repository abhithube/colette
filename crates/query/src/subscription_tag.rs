use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert};

pub enum SubscriptionTag {
    Table,
    SubscriptionId,
    TagId,
    UserId,
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
            }
        )
        .unwrap();
    }
}

pub struct SubscriptionTagInsert<I> {
    pub subscription_id: Uuid,
    pub user_id: Uuid,
    pub tag_ids: I,
}

impl<I: IntoIterator<Item = Uuid>> IntoInsert for SubscriptionTagInsert<I> {
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

        for tag_id in self.tag_ids {
            query.values_panic([
                self.subscription_id.into(),
                tag_id.into(),
                self.user_id.into(),
            ]);
        }

        query
    }
}

pub struct SubscriptionTagDelete<I> {
    pub subscription_id: Uuid,
    pub tag_ids: I,
}

impl<I: IntoIterator<Item = Uuid>> IntoDelete for SubscriptionTagDelete<I> {
    fn into_delete(self) -> DeleteStatement {
        let mut query = Query::delete()
            .from_table(SubscriptionTag::Table)
            .and_where(Expr::col(SubscriptionTag::SubscriptionId).eq(self.subscription_id))
            .to_owned();

        let it = self.tag_ids.into_iter().collect::<Vec<_>>();
        if !it.is_empty() {
            query.and_where(Expr::col(SubscriptionTag::TagId).is_not_in(it));
        }

        query
    }
}
