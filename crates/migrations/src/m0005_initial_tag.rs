use sea_orm::DatabaseBackend;
use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m0002_initial_user::Profile, m0003_initial_profile_feed::ProfileFeed,
    m0004_initial_bookmark::Bookmark, postgres, sqlite,
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

        manager
            .create_table(
                Table::create()
                    .table(ProfileFeedTag::Table)
                    .if_not_exists()
                    .col(uuid(ProfileFeedTag::ProfileFeedId))
                    .col(uuid(ProfileFeedTag::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)
                            .to(ProfileFeed::Table, ProfileFeed::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
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
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BookmarkTag::Table)
                    .if_not_exists()
                    .col(uuid(BookmarkTag::BookmarkId))
                    .col(uuid(BookmarkTag::TagId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(BookmarkTag::Table, BookmarkTag::BookmarkId)
                            .to(Bookmark::Table, Bookmark::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(BookmarkTag::Table, BookmarkTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(BookmarkTag::BookmarkId)
                            .col(BookmarkTag::TagId),
                    )
                    .to_owned(),
            )
            .await?;

        match manager.get_database_backend() {
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_trigger(manager, "tag").await?;
            }
            DatabaseBackend::Sqlite => {
                sqlite::create_updated_at_trigger(manager, "tag").await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BookmarkTag::Table).to_owned())
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

#[derive(DeriveIden)]
pub enum Tag {
    Table,
    Id,
    Title,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
pub enum ProfileFeedTag {
    Table,
    ProfileFeedId,
    TagId,
}

#[derive(DeriveIden)]
pub enum BookmarkTag {
    Table,
    BookmarkId,
    TagId,
}
