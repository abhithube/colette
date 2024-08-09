use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{m0002_initial_user::Profile, m0004_initial_bookmark::Bookmark, postgres};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProfileBookmark::Table)
                    .if_not_exists()
                    .col(uuid(ProfileBookmark::Id).primary_key())
                    .col(uuid(ProfileBookmark::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileBookmark::Table, ProfileBookmark::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(ProfileBookmark::BookmarkId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileBookmark::Table, ProfileBookmark::BookmarkId)
                            .to(Bookmark::Table, Bookmark::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileBookmark::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileBookmark::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("profile_bookmark_profile_id_bookmark_id_idx")
                    .table(ProfileBookmark::Table)
                    .if_not_exists()
                    .col(ProfileBookmark::ProfileId)
                    .col(ProfileBookmark::BookmarkId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == DatabaseBackend::Postgres {
            postgres::create_updated_at_trigger(manager, "profile_bookmark").await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProfileBookmark::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ProfileBookmark {
    Table,
    Id,
    ProfileId,
    BookmarkId,
    CreatedAt,
    UpdatedAt,
}
