use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::account::AccountParams;
use sea_query::{Asterisk, Expr, Iden, InsertStatement, OnConflict, Query, SelectStatement};
use uuid::Uuid;

use crate::{IntoInsert, IntoSelect};

pub enum Account {
    Table,
    Id,
    Sub,
    Provider,
    PasswordHash,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Account {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "accounts",
                Self::Id => "id",
                Self::Sub => "sub",
                Self::Provider => "provider",
                Self::PasswordHash => "password_hash",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for AccountParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column(Asterisk)
            .from(Account::Table)
            .apply_if(self.id, |query, id| {
                query.and_where(Expr::col((Account::Table, Account::Id)).eq(id));
            })
            .apply_if(self.sub, |query, sub| {
                query.and_where(Expr::col((Account::Table, Account::Sub)).eq(sub));
            })
            .apply_if(self.provider, |query, provider| {
                query.and_where(Expr::col((Account::Table, Account::Provider)).eq(provider));
            })
            .to_owned()
    }
}

pub struct AccountInsert<'a> {
    pub id: Uuid,
    pub sub: &'a str,
    pub provider: &'a str,
    pub password_hash: Option<&'a str>,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IntoInsert for AccountInsert<'_> {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Account::Table)
            .columns([
                Account::Id,
                Account::Sub,
                Account::Provider,
                Account::PasswordHash,
                Account::UserId,
                Account::CreatedAt,
                Account::UpdatedAt,
            ])
            .values_panic([
                self.id.into(),
                self.sub.into(),
                self.provider.into(),
                self.password_hash.into(),
                self.user_id.into(),
                self.created_at.into(),
                self.updated_at.into(),
            ])
            .on_conflict(
                OnConflict::column(Account::Id)
                    .update_columns([
                        Account::Sub,
                        Account::Provider,
                        Account::PasswordHash,
                        Account::UpdatedAt,
                    ])
                    .to_owned(),
            )
            .to_owned()
    }
}
