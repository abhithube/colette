use colette_core::collection::{
    CollectionCreateParams, CollectionDeleteParams, CollectionFindByIdParams, CollectionFindParams,
    CollectionUpdateParams,
};
use colette_model::collections;
use sea_query::{
    Asterisk, DeleteStatement, Expr, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

impl IntoSelect for CollectionFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(collections::Entity)
            .apply_if(self.id, |query, id| {
                query.and_where(
                    Expr::col((collections::Entity, collections::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((collections::Entity, collections::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((collections::Entity, collections::Column::Title))
                        .gt(Expr::val(cursor.title)),
                );
            })
            .order_by(
                (collections::Entity, collections::Column::Title),
                Order::Asc,
            )
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
            .column((collections::Entity, collections::Column::Id))
            .column((collections::Entity, collections::Column::UserId))
            .from(collections::Entity)
            .and_where(
                Expr::col((collections::Entity, collections::Column::Id)).eq(self.id.to_string()),
            )
            .to_owned()
    }
}

impl IntoInsert for CollectionCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([
                collections::Column::Id,
                collections::Column::Title,
                collections::Column::FilterRaw,
                collections::Column::UserId,
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
            .table(collections::Entity)
            .and_where(Expr::col(collections::Column::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(collections::Column::Title, title);
        }
        if let Some(filter) = self.filter {
            query.value(
                collections::Column::FilterRaw,
                serde_json::to_string(&filter).unwrap(),
            );
        }

        query
    }
}

impl IntoDelete for CollectionDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(collections::Entity)
            .and_where(Expr::col(collections::Column::Id).eq(self.id.to_string()))
            .to_owned()
    }
}
