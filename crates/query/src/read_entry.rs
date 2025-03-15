use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Query};
use uuid::Uuid;

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

pub struct ReadEntryInsert {
    pub feed_entry_id: Uuid,
    pub subscription_id: Uuid,
    pub user_id: Uuid,
}

impl IntoInsert for ReadEntryInsert {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(ReadEntry::Table)
            .columns([
                ReadEntry::SubscriptionId,
                ReadEntry::FeedEntryId,
                ReadEntry::UserId,
            ])
            .values_panic([
                self.subscription_id.into(),
                self.feed_entry_id.into(),
                self.user_id.into(),
            ])
            .on_conflict(
                OnConflict::columns([ReadEntry::SubscriptionId, ReadEntry::FeedEntryId])
                    .do_nothing()
                    .to_owned(),
            )
            .to_owned()
    }
}

pub struct ReadEntryDelete {
    pub feed_entry_id: Uuid,
    pub subscription_id: Uuid,
}

impl IntoDelete for ReadEntryDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(ReadEntry::Table)
            .and_where(Expr::col(ReadEntry::SubscriptionId).eq(self.subscription_id))
            .and_where(Expr::col(ReadEntry::FeedEntryId).eq(self.feed_entry_id))
            .to_owned()
    }
}
