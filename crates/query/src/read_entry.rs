use colette_core::subscription::SubscriptionEntryUpdateParams;
use colette_model::read_entries;
use sea_query::{DeleteStatement, Expr, InsertStatement, OnConflict, Query};

use crate::{IntoDelete, IntoInsert};

impl IntoInsert for SubscriptionEntryUpdateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(read_entries::Entity)
            .columns([
                read_entries::Column::SubscriptionId,
                read_entries::Column::FeedEntryId,
                read_entries::Column::UserId,
            ])
            .values_panic([
                self.subscription_id.to_string().into(),
                self.feed_entry_id.to_string().into(),
                self.user_id.to_string().into(),
            ])
            .on_conflict(
                OnConflict::columns([
                    read_entries::Column::SubscriptionId,
                    read_entries::Column::FeedEntryId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned()
    }
}

impl IntoDelete for SubscriptionEntryUpdateParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(read_entries::Entity)
            .and_where(
                Expr::col(read_entries::Column::SubscriptionId)
                    .eq(self.subscription_id.to_string()),
            )
            .and_where(
                Expr::col(read_entries::Column::FeedEntryId).eq(self.feed_entry_id.to_string()),
            )
            .to_owned()
    }
}
