use sea_query::{
    ColumnDef, ColumnType, Expr, InsertStatement, Order, Query, SelectStatement, Table,
    TableCreateStatement,
};
use uuid::Uuid;

use crate::common::WithTimestamps;

#[derive(sea_query::Iden)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(User::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(User::Id, id_type)
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new_with_type(User::Email, ColumnType::Text)
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new_with_type(User::Password, ColumnType::Text).not_null())
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn select(id: Option<Uuid>, email: Option<String>) -> SelectStatement {
    Query::select()
        .columns([User::Id, User::Email, User::Password])
        .from(User::Table)
        .and_where_option(id.map(|e| Expr::col((User::Table, User::Id)).eq(e)))
        .and_where_option(email.map(|e| Expr::col((User::Table, User::Email)).eq(e)))
        .order_by((User::Table, User::Email), Order::Asc)
        .to_owned()
}

pub fn insert(id: Uuid, email: String, password: String) -> InsertStatement {
    Query::insert()
        .into_table(User::Table)
        .columns([User::Id, User::Email, User::Password])
        .values_panic([id.into(), email.into(), password.into()])
        .returning_all()
        .to_owned()
}
