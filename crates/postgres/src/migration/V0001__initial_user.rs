use colette_sql::{profile, user};
use sea_query::{ColumnType, PostgresQueryBuilder};

pub fn migration() -> String {
    [
        user::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        profile::create_table(ColumnType::Uuid, ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        profile::create_user_id_is_default_index().build(PostgresQueryBuilder),
        profile::create_user_id_title_index().build(PostgresQueryBuilder),
    ]
    .join("; ")
}
