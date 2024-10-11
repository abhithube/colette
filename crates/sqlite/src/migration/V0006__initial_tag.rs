use colette_sql::{profile_bookmark_tag, profile_feed_tag, tag};
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [
        tag::create_table(ColumnType::Text, ColumnType::Text)
            .build(SqliteQueryBuilder),
        tag::create_profile_id_title_index().build(SqliteQueryBuilder),
        profile_feed_tag::create_table(ColumnType::Text, ColumnType::Text)
            .build(SqliteQueryBuilder),
        profile_bookmark_tag::create_table(ColumnType::Text, ColumnType::Text)
            .build(SqliteQueryBuilder),
    ]
    .join("; ")
}
