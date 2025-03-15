use std::fmt::Write;

use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect};

pub enum Collection {
    Table,
    Id,
    Title,
    FilterRaw,
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
                Self::FilterRaw => "filter_raw",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub struct CollectionSelect<'a> {
    pub id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub cursor: Option<&'a str>,
    pub limit: Option<u64>,
}

impl IntoSelect for CollectionSelect<'_> {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Collection::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Collection::Table, Collection::Id)).eq(id));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Collection::Table, Collection::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, title| {
                query.and_where(
                    Expr::col((Collection::Table, Collection::Title)).gt(Expr::val(title)),
                );
            })
            .order_by((Collection::Table, Collection::Title), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub struct CollectionSelectOne {
    pub id: Uuid,
}

impl IntoSelect for CollectionSelectOne {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(Collection::Table)
            .and_where(Expr::col(Collection::Id).eq(self.id))
            .to_owned()
    }
}

pub struct CollectionInsert<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub filter_raw: &'a str,
    pub user_id: Uuid,
    pub upsert: bool,
}

impl IntoInsert for CollectionInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        let mut query = Query::insert()
            .columns([
                Collection::Id,
                Collection::Title,
                Collection::FilterRaw,
                Collection::UserId,
            ])
            .values_panic([
                self.id.into(),
                self.title.into(),
                self.filter_raw.into(),
                self.user_id.into(),
            ])
            .to_owned();

        if self.upsert {
            query.on_conflict(
                OnConflict::column(Collection::Id)
                    .update_columns([Collection::Title, Collection::FilterRaw])
                    .value(Collection::UpdatedAt, Expr::current_timestamp())
                    .to_owned(),
            );
        }

        query
    }
}

pub struct CollectionDelete {
    pub id: Uuid,
}

impl IntoDelete for CollectionDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Collection::Table)
            .and_where(Expr::col(Collection::Id).eq(self.id))
            .to_owned()
    }
}
