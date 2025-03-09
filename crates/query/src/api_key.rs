use std::fmt::Write;

use colette_core::api_key::{
    ApiKeyCreateParams, ApiKeyDeleteParams, ApiKeyFindByIdParams, ApiKeyFindParams,
    ApiKeySearchParams, ApiKeyUpdateParams,
};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, Order, Query, SelectStatement,
    UpdateStatement,
};

use crate::{IntoDelete, IntoInsert, IntoSelect, IntoUpdate};

pub enum ApiKey {
    Table,
    Id,
    LookupHash,
    VerificationHash,
    Title,
    Preview,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for ApiKey {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "api_keys",
                Self::Id => "id",
                Self::LookupHash => "lookup_hash",
                Self::VerificationHash => "verification_hash",
                Self::Title => "title",
                Self::Preview => "preview",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for ApiKeyFindParams {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(ApiKey::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((ApiKey::Table, ApiKey::Id)).eq(id.to_string()));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((ApiKey::Table, ApiKey::UserId)).eq(user_id.to_string()));
            })
            .apply_if(self.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((ApiKey::Table, ApiKey::CreatedAt))
                        .gt(Expr::val(cursor.created_at.timestamp())),
                );
            })
            .order_by((ApiKey::Table, ApiKey::CreatedAt), Order::Asc)
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
            .column((ApiKey::Table, ApiKey::Id))
            .column((ApiKey::Table, ApiKey::UserId))
            .from(ApiKey::Table)
            .and_where(Expr::col((ApiKey::Table, ApiKey::Id)).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoInsert for ApiKeyCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .columns([
                ApiKey::Id,
                ApiKey::LookupHash,
                ApiKey::VerificationHash,
                ApiKey::Title,
                ApiKey::Preview,
                ApiKey::UserId,
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
            .table(ApiKey::Table)
            .and_where(Expr::col(ApiKey::Id).eq(self.id.to_string()))
            .to_owned();

        if let Some(title) = self.title {
            query.value(ApiKey::Title, title);
        }

        query
    }
}

impl IntoDelete for ApiKeyDeleteParams {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(ApiKey::Table)
            .and_where(Expr::col(ApiKey::Id).eq(self.id.to_string()))
            .to_owned()
    }
}

impl IntoSelect for ApiKeySearchParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((ApiKey::Table, ApiKey::VerificationHash))
            .column((ApiKey::Table, ApiKey::UserId))
            .from(ApiKey::Table)
            .and_where(Expr::col((ApiKey::Table, ApiKey::LookupHash)).eq(self.lookup_hash))
            .to_owned()
    }
}
