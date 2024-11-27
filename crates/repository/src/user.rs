use std::fmt::Write;

use sea_query::{Expr, Iden, InsertStatement, Order, Query, SelectStatement, SimpleExpr};
use uuid::Uuid;

#[allow(dead_code)]
pub enum User {
    Table,
    Id,
    Email,
    Password,
    CreatedAt,
    UpdatedAt,
}

impl Iden for User {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "users",
                Self::Id => "id",
                Self::Email => "email",
                Self::Password => "password",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
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

pub fn insert(id: Option<Uuid>, email: String, password: String) -> InsertStatement {
    let mut columns = vec![User::Email, User::Password];
    let mut values: Vec<SimpleExpr> = vec![email.into(), password.into()];

    if let Some(id) = id {
        columns.push(User::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(User::Table)
        .columns(columns)
        .values_panic(values)
        .returning_col(User::Id)
        .to_owned()
}
