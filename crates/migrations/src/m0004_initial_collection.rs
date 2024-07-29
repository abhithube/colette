use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{m0002_initial_user::Profile, postgres, sqlite};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Collection::Table)
                    .if_not_exists()
                    .col(uuid(Collection::Id).primary_key())
                    .col(text(Collection::Title))
                    .col(boolean(Collection::IsDefault).default(Expr::value(false)))
                    .col(uuid(Collection::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Collection::Table, Collection::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Collection::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Collection::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                "
CREATE UNIQUE INDEX collections_profile_id_is_default_key
    ON collections (profile_id, is_default)
 WHERE is_default",
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Bookmark::Table)
                    .if_not_exists()
                    .col(uuid(Bookmark::Id).primary_key())
                    .col(text(Bookmark::Link))
                    .col(text_null(Bookmark::Title))
                    .col(text_null(Bookmark::ThumbnailUrl))
                    .col(timestamp_with_time_zone_null(Bookmark::PublishedAt))
                    .col(text_null(Bookmark::Author))
                    .col(text(Bookmark::OriginalTitle))
                    .col(text_null(Bookmark::OriginalThumbnailUrl))
                    .col(timestamp_with_time_zone_null(Bookmark::OriginalPublishedAt))
                    .col(text_null(Bookmark::OriginalAuthor))
                    .col(uuid(Bookmark::CollectionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Bookmark::Table, Bookmark::CollectionId)
                            .to(Collection::Table, Collection::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
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

        manager
            .create_index(
                Index::create()
                    .name("bookmarks_collection_id_link_key")
                    .table(Bookmark::Table)
                    .if_not_exists()
                    .col(Bookmark::CollectionId)
                    .col(Bookmark::Link)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, "collections").await?;
                postgres::create_updated_at_trigger(manager, "bookmarks").await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(manager, "collections").await?;
                sqlite::create_updated_at_trigger(manager, "bookmarks").await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Bookmark::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Collection::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Collection {
    #[sea_orm(iden = "collections")]
    Table,
    Id,
    Title,
    IsDefault,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Bookmark {
    #[sea_orm(iden = "bookmarks")]
    Table,
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    OriginalTitle,
    OriginalThumbnailUrl,
    OriginalPublishedAt,
    OriginalAuthor,
    CollectionId,
    CreatedAt,
    UpdatedAt,
}
