use colette_sql::{profile_bookmark_tag, profile_feed_tag, tag};
use sea_query::{ColumnType, PostgresQueryBuilder};

pub fn migration() -> String {
    [
        tag::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        tag::create_profile_id_title_index().build(PostgresQueryBuilder),
        profile_feed_tag::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        profile_bookmark_tag::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
