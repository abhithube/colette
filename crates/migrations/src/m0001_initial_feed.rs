use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};
use strum::IntoEnumIterator;

use crate::{postgres, sqlite};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Feed::Table)
                    .if_not_exists()
                    .col(pk_auto(Feed::Id))
                    .col(text_uniq(Feed::Link))
                    .col(text(Feed::Title))
                    .col(text_null(Feed::Url))
                    .col(
                        timestamp_with_time_zone(Feed::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Feed::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Entry::Table)
                    .if_not_exists()
                    .col(pk_auto(Entry::Id))
                    .col(text_uniq(Entry::Link))
                    .col(text(Entry::Title))
                    .col(timestamp_with_time_zone_null(Entry::PublishedAt))
                    .col(text_null(Entry::Description))
                    .col(text_null(Entry::Author))
                    .col(text_null(Entry::ThumbnailUrl))
                    .col(
                        timestamp_with_time_zone(Entry::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Entry::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FeedEntry::Table)
                    .if_not_exists()
                    .col(pk_auto(FeedEntry::Id))
                    .col(integer(FeedEntry::FeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(FeedEntry::Table, FeedEntry::FeedId)
                            .to(Feed::Table, Feed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(FeedEntry::EntryId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(FeedEntry::Table, FeedEntry::EntryId)
                            .to(Entry::Table, Entry::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(FeedEntry::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(FeedEntry::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let feed_entry_feed_id_entry_id_idx = format!(
            "{feed_entry}_{feed_id}_{entry_id}_idx",
            feed_entry = FeedEntry::Table.to_string(),
            feed_id = FeedEntry::FeedId.to_string(),
            entry_id = FeedEntry::EntryId.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(feed_entry_feed_id_entry_id_idx)
                    .table(FeedEntry::Table)
                    .if_not_exists()
                    .col(FeedEntry::FeedId)
                    .col(FeedEntry::EntryId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, Feed::Table.to_string()).await?;
                postgres::create_updated_at_trigger(manager, Entry::Table.to_string()).await?;
                postgres::create_updated_at_trigger(manager, FeedEntry::Table.to_string()).await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(
                    manager,
                    Feed::Table.to_string(),
                    Feed::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    Entry::Table.to_string(),
                    Entry::iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    FeedEntry::Table.to_string(),
                    FeedEntry::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeedEntry::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Entry::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Feed::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum Feed {
    #[strum(disabled)]
    Table,
    #[strum(disabled)]
    Id,
    Link,
    Title,
    Url,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum Entry {
    #[strum(disabled)]
    Table,
    #[strum(disabled)]
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum FeedEntry {
    #[strum(disabled)]
    Table,
    #[strum(disabled)]
    Id,
    FeedId,
    EntryId,
    CreatedAt,
    UpdatedAt,
}
