use std::fmt::Write;

use colette_core::collection::Cursor;
use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, Order, Query, SelectStatement, SimpleExpr,
    UpdateStatement,
};
use uuid::Uuid;

#[allow(dead_code)]
pub enum Collection {
    Table,
    Id,
    Title,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Collection {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "collections",
                Self::Id => "id",
                Self::Title => "title",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub fn select(
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> SelectStatement {
    let mut query = Query::select()
        .columns([Collection::Id, Collection::Title])
        .from(Collection::Table)
        .and_where(Expr::col(Collection::UserId).eq(user_id))
        .and_where_option(id.map(|e| Expr::col(Collection::Id).eq(e)))
        .and_where_option(cursor.map(|e| Expr::col(Collection::Title).gt(e.title)))
        .order_by((Collection::Table, Collection::Title), Order::Asc)
        .to_owned();

    if let Some(limit) = limit {
        query.limit(limit);
    }

    query
}

pub fn insert(id: Option<Uuid>, title: String, user_id: Uuid) -> InsertStatement {
    let mut columns = vec![Collection::Title, Collection::UserId];
    let mut values: Vec<SimpleExpr> = vec![title.into(), user_id.into()];

    if let Some(id) = id {
        columns.push(Collection::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(Collection::Table)
        .columns(columns)
        .values_panic(values)
        .returning_col(Collection::Id)
        .to_owned()
}

pub fn update(id: Uuid, user_id: Uuid, title: Option<String>) -> UpdateStatement {
    let mut query = Query::update()
        .table(Collection::Table)
        .value(Collection::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(Collection::Id).eq(id))
        .and_where(Expr::col(Collection::UserId).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(Collection::Title, title);
    }

    query
}

pub fn delete_by_id(id: Uuid, user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(Collection::Table)
        .and_where(Expr::col(Collection::Id).eq(id))
        .and_where(Expr::col(Collection::UserId).eq(user_id))
        .to_owned()
}
