use colette_sql::profile_bookmark;
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [
        profile_bookmark::create_table(ColumnType::Text, ColumnType::Text)
            .build(SqliteQueryBuilder),
        profile_bookmark::create_profile_id_bookmark_id_index().build(SqliteQueryBuilder),
    ]
    .join("; ")
}
