use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

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
                    .col(big_integer(Feed::Id).primary_key().auto_increment())
                    .col(text_uniq(Feed::Link))
                    .col(text(Feed::Title))
                    .col(text_null(Feed::Url))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Entry::Table)
                    .if_not_exists()
                    .col(big_integer(Entry::Id).primary_key().auto_increment())
                    .col(text_uniq(Entry::Link))
                    .col(text(Entry::Title))
                    .col(timestamp_with_time_zone_null(Entry::PublishedAt))
                    .col(text_null(Entry::Description))
                    .col(text_null(Entry::Author))
                    .col(text_null(Entry::ThumbnailUrl))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FeedEntry::Table)
                    .if_not_exists()
                    .col(big_integer(FeedEntry::Id).primary_key().auto_increment())
                    .col(big_integer(FeedEntry::FeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(FeedEntry::Table, FeedEntry::FeedId)
                            .to(Feed::Table, Feed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(big_integer(FeedEntry::EntryId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(FeedEntry::Table, FeedEntry::EntryId)
                            .to(Entry::Table, Entry::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("feed_entries_feed_id_entry_id_key")
                    .table(FeedEntry::Table)
                    .if_not_exists()
                    .col(FeedEntry::FeedId)
                    .col(FeedEntry::EntryId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == DatabaseBackend::Postgres {
            #[cfg(feature = "postgres")]
            crate::postgres::create_updated_at_fn(manager).await?;
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

#[derive(DeriveIden)]
pub enum Feed {
    #[sea_orm(iden = "feeds")]
    Table,
    Id,
    Link,
    Title,
    Url,
}

#[derive(DeriveIden)]
pub enum Entry {
    #[sea_orm(iden = "entries")]
    Table,
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
}

#[derive(DeriveIden)]
pub enum FeedEntry {
    #[sea_orm(iden = "feed_entries")]
    Table,
    Id,
    FeedId,
    EntryId,
}
