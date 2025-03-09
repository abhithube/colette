use colette_core::stream::{
    StreamCreateParams, StreamDeleteParams, StreamFindByIdParams, StreamFindParams,
    StreamUpdateParams,
};
use colette_model::streams;
use sea_query::{
    Asterisk, DeleteStatement, Expr, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

impl IntoSelect for StreamFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(streams::Entity)
            .apply_if(self.id, |query, id| {
                query.and_where(
                    Expr::col((streams::Entity, streams::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((streams::Entity, streams::Column::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((streams::Entity, streams::Column::Title))
                        .gt(Expr::val(cursor.title)),
                );
            })
            .order_by((streams::Entity, streams::Column::Title), Order::Asc)
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
            .column((streams::Entity, streams::Column::Id))
            .column((streams::Entity, streams::Column::UserId))
            .from(streams::Entity)
            .and_where(Expr::col((streams::Entity, streams::Column::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for StreamCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([
                streams::Column::Id,
                streams::Column::Title,
                streams::Column::FilterRaw,
                streams::Column::UserId,
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

impl IntoUpdate for StreamUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(streams::Entity)
            .and_where(Expr::col(streams::Column::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(streams::Column::Title, title);
        }
        if let Some(filter) = self.filter {
            query.value(
                streams::Column::FilterRaw,
                serde_json::to_string(&filter).unwrap(),
            );
        }

        query
    }
}

impl IntoDelete for StreamDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(streams::Entity)
            .and_where(Expr::col(streams::Column::Id).eq(self.id.to_string()))
            .to_owned()
    }
}
