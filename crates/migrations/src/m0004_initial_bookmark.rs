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
                    .table(Bookmark::Table)
                    .if_not_exists()
                    .col(integer(Bookmark::Id).primary_key().auto_increment())
                    .col(text_uniq(Bookmark::Link))
                    .col(text(Bookmark::Title))
                    .col(text_null(Bookmark::ThumbnailUrl))
                    .col(timestamp_with_time_zone_null(Bookmark::PublishedAt))
                    .col(text_null(Bookmark::Author))
                    .col(
                        timestamp_with_time_zone(Bookmark::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Bookmark::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, Bookmark::Table.to_string()).await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(
                    manager,
                    Bookmark::Table.to_string(),
                    Bookmark::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bookmark::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum Bookmark {
    #[strum(disabled)]
    Table,
    #[strum(disabled)]
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    CreatedAt,
    UpdatedAt,
}
