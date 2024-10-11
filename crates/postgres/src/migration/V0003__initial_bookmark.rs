use colette_sql::bookmark;
use sea_query::{ColumnType, PostgresQueryBuilder};

pub fn migration() -> String {
    [bookmark::create_table(ColumnType::TimestampWithTimeZone).build(PostgresQueryBuilder)]
        .join("; ")
}
