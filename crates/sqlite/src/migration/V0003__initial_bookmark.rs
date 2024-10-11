use colette_sql::bookmark;
use sea_query::{ColumnType, SqliteQueryBuilder};

pub fn migration() -> String {
    [bookmark::create_table(ColumnType::Text).build(SqliteQueryBuilder)].join("; ")
}
