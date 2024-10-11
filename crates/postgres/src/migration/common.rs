use sea_query::{Alias, ColumnDef, ColumnType, Expr, TableCreateStatement};

pub trait WithTimestamps {
    fn with_timestamps(&mut self, col_type: ColumnType) -> &mut Self;
}

impl WithTimestamps for TableCreateStatement {
    fn with_timestamps(&mut self, col_type: ColumnType) -> &mut Self {
        self.col(
            ColumnDef::new_with_type(Alias::new("created_at"), col_type.clone())
                .not_null()
                .default(Expr::current_timestamp()),
        )
        .col(
            ColumnDef::new_with_type(Alias::new("updated_at"), col_type)
                .not_null()
                .default(Expr::current_timestamp()),
        )
    }
}
