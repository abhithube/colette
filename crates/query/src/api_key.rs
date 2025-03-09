use colette_core::api_key::{
    ApiKeyCreateParams, ApiKeyDeleteParams, ApiKeyFindByIdParams, ApiKeyFindParams,
    ApiKeySearchParams, ApiKeyUpdateParams,
};
use colette_model::api_keys;
use sea_query::{
    Asterisk, DeleteStatement, Expr, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

impl IntoSelect for ApiKeyFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(api_keys::Entity)
            .apply_if(self.id, |query, id| {
                query.and_where(
                    Expr::col((api_keys::Entity, api_keys::Column::Id)).eq(id.to_string()),
                );
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((api_keys::Entity, api_keys::Column::UserId)).eq(user_id.to_string()),
                );
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((api_keys::Entity, api_keys::Column::CreatedAt))
                        .gt(Expr::val(cursor.created_at.timestamp())),
                );
            })
            .order_by((api_keys::Entity, api_keys::Column::CreatedAt), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit as u64);
        }

        query
    }
}

impl IntoSelect for ApiKeyFindByIdParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((api_keys::Entity, api_keys::Column::Id))
            .column((api_keys::Entity, api_keys::Column::UserId))
            .from(api_keys::Entity)
            .and_where(Expr::col((api_keys::Entity, api_keys::Column::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for ApiKeyCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([
                api_keys::Column::Id,
                api_keys::Column::LookupHash,
                api_keys::Column::VerificationHash,
                api_keys::Column::Title,
                api_keys::Column::Preview,
                api_keys::Column::UserId,
            ])
            .values_panic([
                self.id.to_string().into(),
                self.lookup_hash.into(),
                self.verification_hash.into(),
                self.title.into(),
                self.preview.into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}

impl IntoUpdate for ApiKeyUpdateParams {
    fn into_update(self) -> UpdateStatement {
        let mut query = Query::update()
            .table(api_keys::Entity)
            .and_where(Expr::col(api_keys::Column::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(api_keys::Column::Title, title);
        }

        query
    }
}

impl IntoDelete for ApiKeyDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(api_keys::Entity)
            .and_where(Expr::col(api_keys::Column::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoSelect for ApiKeySearchParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((api_keys::Entity, api_keys::Column::VerificationHash))
            .column((api_keys::Entity, api_keys::Column::UserId))
            .from(api_keys::Entity)
            .and_where(
                Expr::col((api_keys::Entity, api_keys::Column::LookupHash)).eq(self.lookup_hash),
            )
            .to_owned()
    }
}
