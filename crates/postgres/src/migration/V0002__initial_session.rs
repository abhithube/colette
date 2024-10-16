use colette_sql::session::Session;
use sea_query::{ColumnDef, PostgresQueryBuilder, Table};

pub fn migration() -> String {
    [Table::create()
        .table(Session::Table)
        .if_not_exists()
        .col(ColumnDef::new(Session::Id).text().not_null().primary_key())
        .col(ColumnDef::new(Session::Data).blob().not_null())
        .col(
            ColumnDef::new(Session::ExpiresAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .build(PostgresQueryBuilder)]
    .join("; ")
}
