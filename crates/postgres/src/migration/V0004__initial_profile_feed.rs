use colette_sql::{profile_feed, profile_feed_entry};
use sea_query::{ColumnType, PostgresQueryBuilder};

pub fn migration() -> String {
    [
        profile_feed::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        profile_feed::create_profile_id_feed_id_index().build(PostgresQueryBuilder),
        profile_feed_entry::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        profile_feed_entry::create_profile_feed_id_feed_entry_id_index()
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
