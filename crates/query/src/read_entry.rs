use std::fmt::Write;

use colette_core::subscription::SubscriptionEntryUpdateParams;
use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};

use crate::{IntoDelete, IntoInsert};

pub enum ReadEntry {
    Table,
    SubscriptionId,
    FeedEntryId,
    UserId,
    CreatedAt,
}

impl Iden for ReadEntry {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "read_entries",
                Self::SubscriptionId => "subscription_id",
                Self::FeedEntryId => "feed_entry_id",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
            }
        )
        .unwrap();
    }
}

impl IntoInsert for SubscriptionEntryUpdateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(ReadEntry::Table)
            .columns([
                ReadEntry::SubscriptionId,
                ReadEntry::FeedEntryId,
                ReadEntry::UserId,
            ])
            .values_panic([
                self.subscription_id.to_string().into(),
                self.feed_entry_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([ReadEntry::SubscriptionId, ReadEntry::FeedEntryId])
                    .do_nothing()
                    .to_owned(),
            )
            .to_owned()
    }
}

impl IntoDelete for SubscriptionEntryUpdateParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(ReadEntry::Table)
            .and_where(Expr::col(ReadEntry::SubscriptionId).eq(self.subscription_id.to_string()))
            .and_where(Expr::col(ReadEntry::FeedEntryId).eq(self.feed_entry_id.to_string()))
            .to_owned()
    }
}
