use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement,
};
use serde_json::Value;
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect};

pub enum Stream {
    Table,
    Id,
    Title,
    FilterJson,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Stream {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "streams",
                Self::Id => "id",
                Self::Title => "title",
                Self::FilterJson => "filter_json",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

#[derive(Default)]
pub struct StreamSelect<'a> {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<&'a str>,
    pub limit: Option<u64>,
}

impl IntoSelect for StreamSelect<'_> {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Stream::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Stream::Table, Stream::Id)).eq(id));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Stream::Table, Stream::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, title| {
                query.and_where(Expr::col((Stream::Table, Stream::Title)).gt(Expr::val(title)));
            })
            .order_by((Stream::Table, Stream::Title), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct StreamInsert<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub filter: Value,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IntoInsert for StreamInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([
                Stream::Id,
                Stream::Title,
                Stream::FilterJson,
                Stream::UserId,
            ])
            .values_panic([
                self.id.into(),
                self.title.into(),
                self.filter.into(),
                self.user_id.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .on_conflict(
                OnConflict::column(Stream::Id)
                    .update_columns([Stream::Title, Stream::FilterJson, Stream::UpdatedAt])
                    .to_owned(),
            )
            .to_owned()
    }
}

pub struct StreamDelete {
    pub id: Uuid,
}

impl IntoDelete for StreamDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Stream::Table)
            .and_where(Expr::col(Stream::Id).eq(self.id))
            .to_owned()
    }
}
