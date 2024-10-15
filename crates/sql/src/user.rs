use sea_query::{Expr, InsertStatement, Order, Query, SelectStatement};
use uuid::Uuid;

#[derive(sea_query::Iden)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    CreatedAt,
    UpdatedAt,
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
