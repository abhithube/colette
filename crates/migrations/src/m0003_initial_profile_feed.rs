use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m0001_initial_feed::{Feed, FeedEntry},
    m0002_initial_user::Profile,
    postgres, sqlite,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProfileFeed::Table)
                    .if_not_exists()
                    .col(uuid(ProfileFeed::Id).primary_key())
                    .col(text_null(ProfileFeed::CustomTitle))
                    .col(uuid(ProfileFeed::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeed::Table, ProfileFeed::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(big_integer(ProfileFeed::FeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeed::Table, ProfileFeed::FeedId)
                            .to(Feed::Table, Feed::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileFeed::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileFeed::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("profile_feeds_profile_id_feed_id_key")
                    .table(ProfileFeed::Table)
                    .if_not_exists()
                    .col(ProfileFeed::ProfileId)
                    .col(ProfileFeed::FeedId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProfileFeedEntry::Table)
                    .if_not_exists()
                    .col(uuid(ProfileFeedEntry::Id).primary_key())
                    .col(boolean(ProfileFeedEntry::HasRead).default(Expr::value(false)))
                    .col(uuid(ProfileFeedEntry::ProfileFeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId)
                            .to(ProfileFeed::Table, ProfileFeed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(big_integer(ProfileFeedEntry::FeedEntryId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId)
                            .to(FeedEntry::Table, FeedEntry::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("profile_feed_entries_profile_feed_id_feed_entry_id_key")
                    .table(ProfileFeedEntry::Table)
                    .if_not_exists()
                    .col(ProfileFeedEntry::ProfileFeedId)
                    .col(ProfileFeedEntry::FeedEntryId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, "profile_feeds").await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(manager, "profile_feeds").await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProfileFeedEntry::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProfileFeed::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum ProfileFeed {
    #[sea_orm(iden = "profile_feeds")]
    Table,
    Id,
    CustomTitle,
    ProfileId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum ProfileFeedEntry {
    #[sea_orm(iden = "profile_feed_entries")]
    Table,
    Id,
    HasRead,
    ProfileFeedId,
    FeedEntryId,
}
