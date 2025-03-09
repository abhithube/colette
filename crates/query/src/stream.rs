use std::fmt::Write;

use colette_core::stream::{
    StreamCreateParams, StreamDeleteParams, StreamFindByIdParams, StreamFindParams,
    StreamUpdateParams,
};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

pub enum Stream {
    Table,
    Id,
    Title,
    FilterRaw,
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
                Self::FilterRaw => "filter_raw",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for StreamFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(Stream::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Stream::Table, Stream::Id)).eq(id.to_string()));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((Stream::Table, Stream::UserId)).eq(user_id.to_string()));
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((Stream::Table, Stream::Title)).gt(Expr::val(cursor.title)),
                );
            })
            .order_by((Stream::Table, Stream::Title), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for StreamFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((Stream::Table, Stream::Id))
            .column((Stream::Table, Stream::UserId))
            .from(Stream::Table)
            .and_where(Expr::col((Stream::Table, Stream::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for StreamCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([Stream::Id, Stream::Title, Stream::FilterRaw, Stream::UserId])
            .values_panic([
                self.id.to_string().into(),
                self.title.clone().into(),
                serde_json::to_string(&self.filter).unwrap().into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for StreamUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(Stream::Table)
            .and_where(Expr::col(Stream::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(Stream::Title, title);
        }
        if let Some(filter) = self.filter {
            query.value(Stream::FilterRaw, serde_json::to_string(&filter).unwrap());
        }

        query
    }
}

impl IntoDelete for StreamDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(Stream::Table)
            .and_where(Expr::col(Stream::Id).eq(self.id.to_string()))
            .to_owned()
    }
}
