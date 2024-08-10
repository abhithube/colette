use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};
use strum::IntoEnumIterator;

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
                    .col(text_null(ProfileFeed::Title))
                    .col(uuid(ProfileFeed::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeed::Table, ProfileFeed::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(ProfileFeed::FeedId))
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
                    .name("profile_feed_profile_id_feed_id_key")
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
                    .col(integer(ProfileFeedEntry::FeedEntryId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId)
                            .to(FeedEntry::Table, FeedEntry::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .col(uuid(ProfileFeedEntry::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        let profile_feed_entry_profile_feed_id_feed_entry_id_idx = format!(
            "{profile_feed_entry}_{profile_feed_id}_{feed_entry_id}_idx",
            profile_feed_entry = ProfileFeedEntry::Table.to_string(),
            profile_feed_id = ProfileFeedEntry::ProfileFeedId.to_string(),
            feed_entry_id = ProfileFeedEntry::FeedEntryId.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(profile_feed_entry_profile_feed_id_feed_entry_id_idx)
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
                postgres::create_updated_at_trigger(manager, ProfileFeed::Table.to_string())
                    .await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(
                    manager,
                    ProfileFeed::Table.to_string(),
                    ProfileFeed::iter()
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
            .drop_table(Table::drop().table(ProfileFeedEntry::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProfileFeed::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum ProfileFeed {
    #[strum(disabled)]
    Table,
    #[strum(disabled)]
    Id,
    Title,
    ProfileId,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum ProfileFeedEntry {
    Table,
    Id,
    HasRead,
    ProfileFeedId,
    FeedEntryId,
    ProfileId,
}
