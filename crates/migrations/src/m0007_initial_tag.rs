use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};
use strum::IntoEnumIterator;

use crate::{
    m0001_initial_user::Profile, m0005_initial_profile_feed::ProfileFeed,
    m0006_initial_collection::ProfileBookmark, postgres, sqlite,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(uuid(Tag::Id).primary_key())
                    .col(text(Tag::Title))
                    .col(uuid(Tag::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Tag::Table, Tag::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Tag::CreatedAt).default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(Tag::UpdatedAt).default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let tag_profile_id_title_idx = format!(
            "{tag}_{profile_id}_{title}_idx",
            tag = Tag::Table.to_string(),
            profile_id = Tag::ProfileId.to_string(),
            title = Tag::Title.to_string()
        );
        manager
            .create_index(
                Index::create()
                    .name(tag_profile_id_title_idx)
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(Tag::ProfileId)
                    .col(Tag::Title)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProfileFeedTag::Table)
                    .if_not_exists()
                    .col(uuid(ProfileFeedTag::ProfileFeedId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)
                            .to(ProfileFeed::Table, ProfileFeed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(ProfileFeedTag::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedTag::Table, ProfileFeedTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(ProfileFeedTag::ProfileFeedId)
                            .col(ProfileFeedTag::TagId),
                    )
                    .col(uuid(ProfileFeedTag::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileFeedTag::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileFeedTag::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ProfileBookmarkTag::Table)
                    .if_not_exists()
                    .col(uuid(ProfileBookmarkTag::ProfileBookmarkId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ProfileBookmarkTag::Table,
                                ProfileBookmarkTag::ProfileBookmarkId,
                            )
                            .to(ProfileBookmark::Table, ProfileBookmark::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(ProfileBookmarkTag::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileBookmarkTag::Table, ProfileBookmarkTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(ProfileBookmarkTag::ProfileBookmarkId)
                            .col(ProfileBookmarkTag::TagId),
                    )
                    .col(uuid(ProfileBookmarkTag::ProfileId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileBookmarkTag::Table, ProfileBookmarkTag::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileBookmarkTag::CreatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        timestamp_with_time_zone(ProfileBookmarkTag::UpdatedAt)
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, Tag::Table.to_string()).await?;
                postgres::create_updated_at_trigger(manager, ProfileFeedTag::Table.to_string())
                    .await?;
                postgres::create_updated_at_trigger(manager, ProfileBookmarkTag::Table.to_string())
                    .await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(
                    manager,
                    Tag::Table.to_string(),
                    Tag::iter().map(|e| e.to_string()).collect::<Vec<_>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    ProfileFeedTag::Table.to_string(),
                    ProfileFeedTag::iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>(),
                )
                .await?;
                sqlite::create_updated_at_trigger(
                    manager,
                    ProfileBookmarkTag::Table.to_string(),
                    ProfileBookmarkTag::iter()
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
            .drop_table(Table::drop().table(ProfileBookmarkTag::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(ProfileFeedTag::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum Tag {
    #[strum(disabled)]
    Table,
    #[strum(disabled)]
    Id,
    Title,
    ProfileId,
    #[strum(disabled)]
    CreatedAt,
    #[strum(disabled)]
    UpdatedAt,
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum ProfileFeedTag {
    #[strum(disabled)]
    Table,
    ProfileFeedId,
    TagId,
    ProfileId,
    #[strum(disabled)]
    CreatedAt,
    #[strum(disabled)]
    UpdatedAt,
}

#[derive(DeriveIden, strum_macros::EnumIter)]
pub enum ProfileBookmarkTag {
    #[strum(disabled)]
    Table,
    ProfileBookmarkId,
    TagId,
    ProfileId,
    #[strum(disabled)]
    CreatedAt,
    #[strum(disabled)]
    UpdatedAt,
}
