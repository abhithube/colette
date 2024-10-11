use colette_sql::{smart_feed, smart_feed_filter};
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [
        smart_feed::create_table(ColumnType::Text, ColumnType::Text).build(SqliteQueryBuilder),
        smart_feed::create_profile_id_title_index().build(SqliteQueryBuilder),
        smart_feed_filter::create_table(
            ColumnType::Text,
            ColumnType::Text,
            ColumnType::Text,
            ColumnType::Text,
        )
        .build(SqliteQueryBuilder),
    ]
    .join("; ")
}
