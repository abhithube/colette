#[allow(unused_imports)]
use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

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
                    .table(User::Table)
                    .if_not_exists()
                    .col(uuid(User::Id).primary_key())
                    .col(text_uniq(User::Email))
                    .col(text(User::Password))
                    .col(
                        timestamp_with_time_zone(User::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(User::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Profile::Table)
                    .if_not_exists()
                    .col(uuid(Profile::Id).primary_key())
                    .col(text(Profile::Title))
                    .col(text_null(Profile::ImageUrl))
                    .col(boolean(Profile::IsDefault).default(Expr::val(false)))
                    .col(uuid(Profile::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Profile::Table, Profile::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Profile::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Profile::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let profile_user_id_is_default_idx = format!(
            r#"
CREATE UNIQUE INDEX {profile}_{user_id}_{is_default}_idx
    ON "{profile}" ("{user_id}", "{is_default}")
 WHERE "{is_default}""#,
            profile = Profile::Table.to_string(),
            user_id = Profile::UserId.to_string(),
            is_default = Profile::IsDefault.to_string()
        );
        manager
            .get_connection()
            .execute_unprepared(&profile_user_id_is_default_idx)
            .await?;

        let profile_user_id_title_idx = format!(
            "{profile}_{user_id}_{title}_idx",
            profile = Profile::Table.to_string(),
            user_id = Profile::UserId.to_string(),
            title = Profile::Title.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(profile_user_id_title_idx)
                    .table(Profile::Table)
                    .if_not_exists()
                    .col(Profile::UserId)
                    .col(Profile::Title)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            #[cfg(feature = "postgres")]
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, User::Table.to_string()).await?;
                postgres::create_updated_at_trigger(manager, Profile::Table.to_string()).await?;
            }
            #[cfg(feature = "sqlite")]
            DatabaseBackend::Sqlite => {
                use strum::IntoEnumIterator;

                sqlite::create_updated_at_trigger(
                    manager,
                    User::Table.to_string(),
                    User::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    Profile::Table.to_string(),
                    Profile::iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>(),
                )
                .await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Profile::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum User {
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Table,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Id,
    Email,
    Password,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    CreatedAt,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    UpdatedAt,
}

#[derive(DeriveIden)]
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum Profile {
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Table,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Id,
    Title,
    ImageUrl,
    IsDefault,
    UserId,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    CreatedAt,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    UpdatedAt,
}
