use colette_sql::{
    smart_feed,
    smart_feed_filter::{self, Field, Operation},
};
use sea_query::{extension::postgres::Type, ColumnType, PostgresQueryBuilder, SeaRc};

pub fn migration() -> String {
    [
        smart_feed::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        smart_feed::create_profile_id_title_index().build(PostgresQueryBuilder),
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
        smart_feed_filter::create_table(
            ColumnType::Uuid,
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
            ColumnType::TimestampWithTimeZone,
        )
        .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
