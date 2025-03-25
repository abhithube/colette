use std::fmt::Write;

use chrono::{DateTime, Utc};
use sea_query::{
    Asterisk, DeleteStatement, Expr, Iden, InsertStatement, OnConflict, Order, Query,
    SelectStatement,
};
use uuid::Uuid;

use crate::{IntoDelete, IntoInsert, IntoSelect};

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

pub struct ApiKeySelect<'a> {
    pub id: Option<Uuid>,
    pub user_id: Option<&'a str>,
    pub cursor: Option<DateTime<Utc>>,
    pub limit: Option<u64>,
}

impl IntoSelect for ApiKeySelect<'_> {
    fn into_select(self) -> SelectStatement {
        let mut query = Query::select()
            .column(Asterisk)
            .from(ApiKey::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((ApiKey::Table, ApiKey::Id)).eq(id));
            })
            .apply_if(self.user_id, |query, user_id| {
                query.and_where(Expr::col((ApiKey::Table, ApiKey::UserId)).eq(user_id));
            })
            .apply_if(self.cursor, |query, created_at| {
                query.and_where(
                    Expr::col((ApiKey::Table, ApiKey::CreatedAt)).gt(Expr::val(created_at)),
                );
            })
            .order_by((ApiKey::Table, ApiKey::CreatedAt), Order::Asc)
            .to_owned();

        if let Some(limit) = self.limit {
            query.limit(limit);
        }

        query
    }
}

pub enum ApiKeySelectOne<'a> {
    Id(Uuid),
    LookupHash(&'a str),
}

impl IntoSelect for ApiKeySelectOne<'_> {
    fn into_select(self) -> SelectStatement {
        let r#where = match self {
            ApiKeySelectOne::Id(id) => Expr::col(ApiKey::Id).eq(id),
            ApiKeySelectOne::LookupHash(lookup_hash) => {
                Expr::col(ApiKey::LookupHash).eq(lookup_hash)
            }
        };

        Query::select()
            .column(Asterisk)
            .from(ApiKey::Table)
            .and_where(r#where)
            .to_owned()
    }
}

pub struct ApiKeyInsert<'a> {
    pub id: Uuid,
    pub lookup_hash: &'a str,
    pub verification_hash: &'a str,
    pub title: &'a str,
    pub preview: &'a str,
    pub user_id: &'a str,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IntoInsert for ApiKeyInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(ApiKey::Table)
            .columns([
                ApiKey::Id,
                ApiKey::LookupHash,
                ApiKey::VerificationHash,
                ApiKey::Title,
                ApiKey::Preview,
                ApiKey::CreatedAt,
                ApiKey::UpdatedAt,
                ApiKey::UserId,
            ])
            .values_panic([
                self.id.into(),
                self.lookup_hash.into(),
                self.verification_hash.into(),
                self.title.into(),
                self.preview.into(),
                self.created_at.into(),
                self.updated_at.into(),
                self.user_id.into(),
            ])
            .on_conflict(
                OnConflict::column(ApiKey::Id)
                    .update_columns([ApiKey::Title, ApiKey::UpdatedAt])
                    .to_owned(),
            )
            .to_owned()
    }
}

pub struct ApiKeyDelete {
    pub id: Uuid,
}

impl IntoDelete for ApiKeyDelete {
    fn into_delete(self) -> DeleteStatement {
        Query::delete()
            .from_table(ApiKey::Table)
            .and_where(Expr::col(ApiKey::Id).eq(self.id))
            .to_owned()
    }
}
