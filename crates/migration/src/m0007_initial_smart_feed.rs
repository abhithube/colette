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
                    .table(SmartFeed::Table)
                    .if_not_exists()
                    .col(uuid(SmartFeed::Id).primary_key())
                    .col(text(SmartFeed::Title))
                    .col(uuid(SmartFeed::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SmartFeed::Table, SmartFeed::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(SmartFeed::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(SmartFeed::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let smart_feed_profile_id_title_idx = format!(
            "{smart_feed}_{profile_id}_{title}_idx",
            smart_feed = SmartFeed::Table.to_string(),
            profile_id = SmartFeed::ProfileId.to_string(),
            title = SmartFeed::Title.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(smart_feed_profile_id_title_idx)
                    .table(SmartFeed::Table)
                    .if_not_exists()
                    .col(SmartFeed::ProfileId)
                    .col(SmartFeed::Title)
                    .unique()
                    .to_owned(),
            )
            .await?;

        #[cfg(feature = "postgres")]
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            use extension::postgres::Type;

            manager
                .create_type(
                    Type::create()
                        .as_enum(Field::Enum)
                        .values([
                            Field::Link,
                            Field::Title,
                            Field::PublishedAt,
                            Field::Description,
                            Field::Author,
                            Field::HasRead,
                        ])
                        .to_owned(),
                )
                .await?;

            manager
                .create_type(
                    Type::create()
                        .as_enum(Operation::Enum)
                        .values([
                            Operation::Eq,
                            Operation::Ne,
                            Operation::Like,
                            Operation::NotLike,
                            Operation::GreaterThan,
                            Operation::LessThan,
                            Operation::InLastXSec,
                        ])
                        .to_owned(),
                )
                .await?;
        }

        manager
            .create_table(
                Table::create()
                    .table(SmartFeedFilter::Table)
                    .if_not_exists()
                    .col(uuid(SmartFeedFilter::Id).primary_key())
                    .col(enumeration(
                        SmartFeedFilter::Field,
                        Field::Enum,
                        [
                            Field::Link,
                            Field::Title,
                            Field::PublishedAt,
                            Field::Description,
                            Field::Author,
                            Field::HasRead,
                        ],
                    ))
                    .col(enumeration(
                        SmartFeedFilter::Operation,
                        Operation::Enum,
                        [
                            Operation::Eq,
                            Operation::Ne,
                            Operation::Like,
                            Operation::NotLike,
                            Operation::GreaterThan,
                            Operation::LessThan,
                            Operation::InLastXSec,
                        ],
                    ))
                    .col(text(SmartFeedFilter::Value))
                    .col(uuid(SmartFeedFilter::SmartFeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId)
                            .to(SmartFeed::Table, SmartFeed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(SmartFeedFilter::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(SmartFeedFilter::Table, SmartFeedFilter::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(SmartFeedFilter::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(SmartFeedFilter::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            #[cfg(feature = "postgres")]
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, SmartFeed::Table.to_string()).await?;
                postgres::create_updated_at_trigger(manager, SmartFeedFilter::Table.to_string())
                    .await?;
            }
            #[cfg(feature = "sqlite")]
            DatabaseBackend::Sqlite => {
                use strum::IntoEnumIterator;

                sqlite::create_updated_at_trigger(
                    manager,
                    SmartFeed::Table.to_string(),
                    SmartFeed::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    SmartFeedFilter::Table.to_string(),
                    SmartFeedFilter::iter()
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
        #[cfg(feature = "postgres")]
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            use extension::postgres::Type;

            manager
                .drop_type(Type::drop().name(Operation::Enum).to_owned())
                .await?;

            manager
                .drop_type(Type::drop().name(Field::Enum).to_owned())
                .await?;
        }

        manager
            .drop_table(Table::drop().table(SmartFeedFilter::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(SmartFeed::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum SmartFeed {
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Table,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Id,
    Title,
    ProfileId,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    CreatedAt,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    UpdatedAt,
}

#[derive(DeriveIden)]
#[cfg_attr(feature = "sqlite", derive(strum_macros::EnumIter))]
pub enum SmartFeedFilter {
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Table,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    Id,
    Field,
    Operation,
    Value,
    SmartFeedId,
    ProfileId,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    CreatedAt,
    #[cfg_attr(feature = "sqlite", strum(disabled))]
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum Field {
    #[sea_orm(iden = "field")]
    Enum,
    #[sea_orm(iden = "link")]
    Link,
    #[sea_orm(iden = "title")]
    Title,
    #[sea_orm(iden = "published_at")]
    PublishedAt,
    #[sea_orm(iden = "description")]
    Description,
    #[sea_orm(iden = "author")]
    Author,
    #[sea_orm(iden = "has_read")]
    HasRead,
}

#[derive(DeriveIden)]
pub enum Operation {
    #[sea_orm(iden = "operation")]
    Enum,
    #[sea_orm(iden = "=")]
    Eq,
    #[sea_orm(iden = "!=")]
    Ne,
    #[sea_orm(iden = "LIKE")]
    Like,
    #[sea_orm(iden = "NOT LIKE")]
    NotLike,
    #[sea_orm(iden = ">")]
    GreaterThan,
    #[sea_orm(iden = "<")]
    LessThan,
    #[sea_orm(iden = "in_last_x_sec")]
    InLastXSec,
}
