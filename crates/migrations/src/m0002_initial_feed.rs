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
                    .table(FeedEntry::Table)
                    .if_not_exists()
                    .col(pk_auto(FeedEntry::Id))
                    .col(text_uniq(FeedEntry::Link))
                    .col(text(FeedEntry::Title))
                    .col(timestamp_with_time_zone_null(FeedEntry::PublishedAt))
                    .col(text_null(FeedEntry::Description))
                    .col(text_null(FeedEntry::Author))
                    .col(text_null(FeedEntry::ThumbnailUrl))
                    .col(integer(FeedEntry::FeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(FeedEntry::Table, FeedEntry::FeedId)
                            .to(Feed::Table, Feed::Id)
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

        let feed_entry_feed_id_link_idx = format!(
            "{feed_entry}_{feed_id}_{link}_idx",
            feed_entry = FeedEntry::Table.to_string(),
            feed_id = FeedEntry::FeedId.to_string(),
            link = FeedEntry::Link.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(feed_entry_feed_id_link_idx)
                    .table(FeedEntry::Table)
                    .if_not_exists()
                    .col(FeedEntry::FeedId)
                    .col(FeedEntry::Link)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, Feed::Table.to_string()).await?;
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
    #[strum(disabled)]
    CreatedAt,
    #[strum(disabled)]
    UpdatedAt,
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum FeedEntry {
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
    FeedId,
    #[strum(disabled)]
    CreatedAt,
    #[strum(disabled)]
    UpdatedAt,
}
