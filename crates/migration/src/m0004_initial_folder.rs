#[allow(unused_imports)]
use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

use crate::m0001_initial_user::Profile;
#[cfg(feature = "postgres")]
use crate::postgres;
#[cfg(feature = "sqlite")]
use crate::sqlite;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Folder::Table)
                    .if_not_exists()
                    .col(uuid(Folder::Id).primary_key())
                    .col(text(Folder::Title))
                    .col(uuid_null(Folder::ParentId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Folder::Table, Folder::ParentId)
                            .to(Folder::Table, Folder::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(Folder::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Folder::Table, Folder::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Folder::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Folder::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let folder_profile_id_parent_id_title_idx = format!(
            "{folder}_{profile_id}_{parent_id}_{title}_idx",
            folder = Folder::Table.to_string(),
            profile_id = Folder::ProfileId.to_string(),
            parent_id = Folder::ParentId.to_string(),
            title = Folder::Title.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(folder_profile_id_parent_id_title_idx)
                    .table(Folder::Table)
                    .if_not_exists()
                    .col(Folder::ProfileId)
                    .col(Folder::ParentId)
                    .col(Folder::Title)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            #[cfg(feature = "postgres")]
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, Folder::Table.to_string()).await?;
            }
            #[cfg(feature = "sqlite")]
            DatabaseBackend::Sqlite => {
                use strum::IntoEnumIterator;

                sqlite::create_updated_at_trigger(
                    manager,
                    Folder::Table.to_string(),
                    Folder::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Folder::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum Folder {
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
