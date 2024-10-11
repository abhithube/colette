use colette_sql::{profile_feed, profile_feed_entry};
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [
        profile_feed::create_table(ColumnType::Text, ColumnType::Text).build(SqliteQueryBuilder),
        profile_feed::create_profile_id_feed_id_index().build(SqliteQueryBuilder),
        profile_feed_entry::create_table(ColumnType::Text, ColumnType::Text)
            .build(SqliteQueryBuilder),
        profile_feed_entry::create_profile_feed_id_feed_entry_id_index().build(SqliteQueryBuilder),
    ]
    .join("; ")
}
