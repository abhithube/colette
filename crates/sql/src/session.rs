use chrono::{DateTime, Utc};
use sea_query::{DeleteStatement, Expr, InsertStatement, OnConflict, Query, SelectStatement};

#[derive(sea_query::Iden)]
pub enum Session {
    Table,
    Id,
    Data,
    ExpiresAt,
}

pub fn select_by_id(id: String) -> SelectStatement {
    Query::select()
        .column(Session::Data)
        .from(Session::Table)
        .and_where(Expr::col(Session::Id).eq(id))
        .and_where(Expr::col(Session::ExpiresAt).gt(Expr::current_timestamp()))
        .to_owned()
}

pub fn insert(id: String, data: &[u8], expires_at: DateTime<Utc>) -> InsertStatement {
    Query::insert()
        .into_table(Session::Table)
        .columns([Session::Id, Session::Data, Session::ExpiresAt])
        .values_panic([id.into(), data.into(), expires_at.into()])
        .to_owned()
}

pub fn upsert(id: String, data: &[u8], expires_at: DateTime<Utc>) -> InsertStatement {
    Query::insert()
        .into_table(Session::Table)
        .columns([Session::Id, Session::Data, Session::ExpiresAt])
        .values_panic([id.into(), data.into(), expires_at.into()])
        .on_conflict(
            OnConflict::column(Session::Id)
                .update_columns([Session::Data, Session::ExpiresAt])
                .to_owned(),
        )
        .to_owned()
}

pub fn delete_by_id(id: String) -> DeleteStatement {
    Query::delete()
        .from_table(Session::Table)
        .and_where(Expr::col((Session::Table, Session::Id)).eq(id))
        .to_owned()
}

pub fn delete_many() -> DeleteStatement {
    Query::delete()
        .from_table(Session::Table)
        .and_where(Expr::col((Session::Table, Session::ExpiresAt)).lt(Expr::current_timestamp()))
        .to_owned()
}
