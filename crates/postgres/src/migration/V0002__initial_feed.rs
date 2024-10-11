use colette_sql::{feed, feed_entry};
use sea_query::{ColumnType, PostgresQueryBuilder};

pub fn migration() -> String {
    [
        feed::create_table(ColumnType::TimestampWithTimeZone).build(PostgresQueryBuilder),
        feed_entry::create_table(ColumnType::TimestampWithTimeZone).build(PostgresQueryBuilder),
        feed_entry::create_feed_id_link_index().build(PostgresQueryBuilder),
    ]
    .join("; ")
}
