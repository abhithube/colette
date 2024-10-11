use colette_sql::profile_bookmark;
use sea_query::{ColumnType, PostgresQueryBuilder};

pub fn migration() -> String {
    [
        profile_bookmark::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        profile_bookmark::create_profile_id_bookmark_id_index().build(PostgresQueryBuilder),
    ]
    .join("; ")
}
