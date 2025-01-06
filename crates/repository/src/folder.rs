use std::fmt::Write;

use colette_core::folder::Cursor;
use sea_query::{
    DeleteStatement, Expr, Iden, InsertStatement, Order, Query, SelectStatement, SimpleExpr,
    UpdateStatement,
};
use uuid::Uuid;

#[allow(dead_code)]
pub enum Folder {
    Table,
    Id,
    Title,
    ParentId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Folder {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "folders",
                Self::Id => "id",
                Self::Title => "title",
                Self::ParentId => "parent_id",
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
    parent_id: Option<Option<Uuid>>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> SelectStatement {
    let mut query = Query::select()
        .columns([Folder::Id, Folder::Title, Folder::ParentId])
        .from(Folder::Table)
        .and_where(Expr::col(Folder::UserId).eq(user_id))
        .and_where_option(id.map(|e| Expr::col(Folder::Id).eq(e)))
        .and_where_option(parent_id.map(|e| Expr::col(Folder::ParentId).eq(e)))
        .and_where_option(cursor.map(|e| Expr::col(Folder::Title).gt(e.title)))
        .order_by((Folder::Table, Folder::Title), Order::Asc)
        .to_owned();

    if let Some(limit) = limit {
        query.limit(limit);
    }

    query
}

pub fn insert(
    id: Option<Uuid>,
    title: String,
    parent_id: Option<Uuid>,
    user_id: Uuid,
) -> InsertStatement {
    let mut columns = vec![Folder::Title, Folder::ParentId, Folder::UserId];
    let mut values: Vec<SimpleExpr> = vec![title.into(), parent_id.into(), user_id.into()];

    if let Some(id) = id {
        columns.push(Folder::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(Folder::Table)
        .columns(columns)
        .values_panic(values)
        .returning_col(Folder::Id)
        .to_owned()
}

pub fn update(
    id: Uuid,
    user_id: Uuid,
    title: Option<String>,
    parent_id: Option<Option<Uuid>>,
) -> UpdateStatement {
    let mut query = Query::update()
        .table(Folder::Table)
        .value(Folder::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(Folder::Id).eq(id))
        .and_where(Expr::col(Folder::UserId).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(Folder::Title, title);
    }
    if let Some(parent_id) = parent_id {
        query.value(Folder::ParentId, parent_id);
    }

    query
}

pub fn delete_by_id(id: Uuid, user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(Folder::Table)
        .and_where(Expr::col(Folder::Id).eq(id))
        .and_where(Expr::col(Folder::UserId).eq(user_id))
        .to_owned()
}
