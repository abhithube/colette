use colette_sql::{profile, user};
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [
        user::create_table(ColumnType::Text, ColumnType::Text).build(SqliteQueryBuilder),
        profile::create_table(ColumnType::Text, ColumnType::Text).build(SqliteQueryBuilder),
        profile::create_user_id_is_default_index().build(SqliteQueryBuilder),
        profile::create_user_id_title_index().build(SqliteQueryBuilder),
    ]
    .join("; ")
}
