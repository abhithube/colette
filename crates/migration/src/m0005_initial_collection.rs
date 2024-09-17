#[allow(unused_imports)]
use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

#[cfg(feature = "postgres")]
use crate::postgres;
#[cfg(feature = "sqlite")]
use crate::sqlite;
use crate::{m0001_initial_user::Profile, m0003_initial_bookmark::Bookmark};

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
                    .col(uuid_null(Collection::ParentId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Collection::Table, Collection::ParentId)
                            .to(Collection::Table, Collection::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
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

        let collection_profile_id_parent_id_title_idx = format!(
            "{collection}_{profile_id}_{parent_id}_{title}_idx",
            collection = Collection::Table.to_string(),
            profile_id = Collection::ProfileId.to_string(),
            parent_id = Collection::ParentId.to_string(),
            title = Collection::Title.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(collection_profile_id_parent_id_title_idx)
                    .table(Collection::Table)
                    .if_not_exists()
                    .col(Collection::ProfileId)
                    .col(Collection::ParentId)
                    .col(Collection::Title)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProfileBookmark::Table)
                    .if_not_exists()
                    .col(uuid(ProfileBookmark::Id).primary_key())
                    .col(unsigned(ProfileBookmark::SortIndex))
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
                    .col(uuid_null(ProfileBookmark::CollectionId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileBookmark::Table, ProfileBookmark::CollectionId)
                            .to(Collection::Table, Collection::Id)
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

        let profile_bookmark_profile_id_bookmark_id_idx = format!(
            "{profile_bookmark}_{profile_id}_{bookmark_id}_idx",
            profile_bookmark = ProfileBookmark::Table.to_string(),
            profile_id = ProfileBookmark::ProfileId.to_string(),
            bookmark_id = ProfileBookmark::BookmarkId.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(profile_bookmark_profile_id_bookmark_id_idx)
                    .table(ProfileBookmark::Table)
                    .if_not_exists()
                    .col(ProfileBookmark::ProfileId)
                    .col(ProfileBookmark::BookmarkId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            #[cfg(feature = "postgres")]
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, Collection::Table.to_string()).await?;
                postgres::create_updated_at_trigger(manager, ProfileBookmark::Table.to_string())
                    .await?;
            }
            #[cfg(feature = "sqlite")]
            DatabaseBackend::Sqlite => {
                use strum::IntoEnumIterator;

                sqlite::create_updated_at_trigger(
                    manager,
                    Collection::Table.to_string(),
                    Collection::iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    ProfileBookmark::Table.to_string(),
                    ProfileBookmark::iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<_>>(),
                )
                .await?;
            }
            _ => {}
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
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum Collection {
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Table,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Id,
    Title,
    ParentId,
    ProfileId,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    CreatedAt,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    UpdatedAt,
}

#[derive(DeriveIden)]
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum ProfileBookmark {
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Table,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Id,
    SortIndex,
    ProfileId,
    BookmarkId,
    CollectionId,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    CreatedAt,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    UpdatedAt,
}