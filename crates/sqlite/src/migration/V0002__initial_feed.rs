use colette_sql::{feed, feed_entry};
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [
        feed::create_table(ColumnType::Text).build(SqliteQueryBuilder),
        feed_entry::create_table(ColumnType::Text).build(SqliteQueryBuilder),
        feed_entry::create_feed_id_link_index().build(SqliteQueryBuilder),
    ]
    .join("; ")
}
