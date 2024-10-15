use sea_query::{Alias, ColumnDef, Expr, TableCreateStatement};

pub trait WithPk {
    fn with_uuid_pk(&mut self) -> &mut Self;

    fn with_integer_pk(&mut self) -> &mut Self;
}

impl WithPk for TableCreateStatement {
    fn with_uuid_pk(&mut self) -> &mut Self {
        self.col(
            ColumnDef::new(Alias::new("id"))
                .uuid()
                .not_null()
                .primary_key(),
        )
    }

    fn with_integer_pk(&mut self) -> &mut Self {
        self.col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .primary_key()
                .auto_increment(),
        )
    }
}

pub trait WithTimestamps {
    fn with_timestamps(&mut self) -> &mut Self;
}

impl WithTimestamps for TableCreateStatement {
    fn with_timestamps(&mut self) -> &mut Self {
        self.col(
            ColumnDef::new(Alias::new("created_at"))
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .col(
            ColumnDef::new(Alias::new("updated_at"))
                .timestamp_with_time_zone()
                .not_null()
                .default(Expr::current_timestamp()),
        )
    }
}
