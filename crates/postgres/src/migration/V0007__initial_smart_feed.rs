use colette_sql::{
    profile::Profile,
    smart_feed::SmartFeed,
    smart_feed_filter::{Field, Operation, SmartFeedFilter},
};
use sea_query::{
    extension::postgres::Type, ColumnDef, ColumnType, ForeignKey, ForeignKeyAction, Iden, Index,
    PostgresQueryBuilder, SeaRc, Table,
};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [
        Table::create()
            .table(SmartFeed::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(SmartFeed::Id, ColumnType::Uuid)
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new_with_type(SmartFeed::Title, ColumnType::Text).not_null())
            .col(ColumnDef::new_with_type(SmartFeed::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(SmartFeed::Table, SmartFeed::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        Index::create()
            .name(format!(
                "{smart_feed}_{profile_id}_{title}_idx",
                smart_feed = SmartFeed::Table.to_string(),
                profile_id = SmartFeed::ProfileId.to_string(),
                title = SmartFeed::Title.to_string()
            ))
            .table(SmartFeed::Table)
            .if_not_exists()
            .col(SmartFeed::ProfileId)
            .col(SmartFeed::Title)
            .unique()
            .build(PostgresQueryBuilder),
        Type::create()
            .as_enum(Field::Type)
            .values([
                Field::Link,
                Field::Title,
                Field::PublishedAt,
                Field::Description,
                Field::Author,
                Field::HasRead,
            ])
            .to_string(PostgresQueryBuilder),
        Type::create()
            .as_enum(Operation::Type)
            .values([
                Operation::Eq,
                Operation::Ne,
                Operation::Like,
                Operation::NotLike,
                Operation::GreaterThan,
                Operation::LessThan,
                Operation::InLastXSec,
            ])
            .to_string(PostgresQueryBuilder),
        Table::create()
            .table(SmartFeedFilter::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(SmartFeedFilter::Id, ColumnType::Uuid)
                    .not_null()
                    .primary_key(),
            )
            .col(
                ColumnDef::new_with_type(
                    SmartFeedFilter::Field,
                    ColumnType::Enum {
                        name: SeaRc::new(Field::Type),
                        variants: vec![
                            SeaRc::new(Field::Link),
                            SeaRc::new(Field::Title),
                            SeaRc::new(Field::PublishedAt),
                            SeaRc::new(Field::Description),
                            SeaRc::new(Field::Author),
                            SeaRc::new(Field::HasRead),
                        ],
                    },
                )
                .not_null(),
            )
            .col(
                ColumnDef::new_with_type(
                    SmartFeedFilter::Operation,
                    ColumnType::Enum {
                        name: SeaRc::new(Operation::Type),
                        variants: vec![
                            SeaRc::new(Operation::Eq),
                            SeaRc::new(Operation::Ne),
                            SeaRc::new(Operation::Like),
                            SeaRc::new(Operation::NotLike),
                            SeaRc::new(Operation::GreaterThan),
                            SeaRc::new(Operation::LessThan),
                            SeaRc::new(Operation::InLastXSec),
                        ],
                    },
                )
                .not_null(),
            )
            .col(ColumnDef::new_with_type(SmartFeedFilter::Value, ColumnType::Text).not_null())
            .col(
                ColumnDef::new_with_type(SmartFeedFilter::SmartFeedId, ColumnType::Uuid).not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId)
                    .to(SmartFeed::Table, SmartFeed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new_with_type(SmartFeedFilter::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(SmartFeedFilter::Table, SmartFeedFilter::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
