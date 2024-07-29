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

        manager
            .get_connection()
            .execute_unprepared(
                "
CREATE UNIQUE INDEX profiles_user_id_is_default_key
    ON profiles (user_id, is_default)
 WHERE is_default",
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                #[cfg(feature = "postgres")]
                {
                    crate::postgres::create_updated_at_trigger(manager, "users").await?;
                    crate::postgres::create_updated_at_trigger(manager, "profiles").await?;
                }
            }
            DatabaseBackend::Sqlite => {
                #[cfg(feature = "sqlite")]
                {
                    crate::sqlite::create_updated_at_trigger(manager, "users").await?;
                    crate::sqlite::create_updated_at_trigger(manager, "profiles").await?;
                }
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
pub enum User {
    #[sea_orm(iden = "users")]
    Table,
    Id,
    Email,
    Password,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Profile {
    #[sea_orm(iden = "profiles")]
    Table,
    Id,
    Title,
    ImageUrl,
    IsDefault,
    UserId,
    CreatedAt,
    UpdatedAt,
}
