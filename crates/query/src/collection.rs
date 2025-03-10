use std::fmt::Write;

use colette_core::collection::{
    CollectionCreateParams, CollectionDeleteParams, CollectionFindByIdParams, CollectionFindParams,
    CollectionUpdateParams,
};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

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

impl IntoSelect for CollectionFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Collection::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Collection::Table, Collection::Id)).eq(id.to_string()));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((Collection::Table, Collection::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((Collection::Table, Collection::Title)).gt(Expr::val(cursor.title)),
                );
            })
            .order_by((Collection::Table, Collection::Title), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for CollectionFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((Collection::Table, Collection::Id))
            .column((Collection::Table, Collection::UserId))
            .from(Collection::Table)
            .and_where(Expr::col((Collection::Table, Collection::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for CollectionCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([
                Collection::Id,
                Collection::Title,
                Collection::FilterRaw,
                Collection::UserId,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.title.clone().into(),
                serde_json::to_string(&self.filter).unwrap().into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for CollectionUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(Collection::Table)
            .value(Collection::UpdatedAt, Expr::current_timestamp())
            .and_where(Expr::col(Collection::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(Collection::Title, title);
        }
        if let Some(filter) = self.filter {
            query.value(
                Collection::FilterRaw,
                serde_json::to_string(&filter).unwrap(),
            );
        }

        query
    }
}

impl IntoDelete for CollectionDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Collection::Table)
            .and_where(Expr::col(Collection::Id).eq(self.id.to_string()))
            .to_owned()
    }
}
