use std::fmt::Write;

use colette_core::account::{AccountCreateParams, AccountFindParams};
use sea_query::{Expr, Iden, InsertStatement, Query, SelectStatement};

use crate::{IntoInsert, IntoSelect, user::User};

pub enum Account {
    Table,
    ProviderId,
    AccountId,
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
                Self::ProviderId => "provider_id",
                Self::AccountId => "account_id",
                Self::PasswordHash => "password_hash",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

impl IntoSelect for AccountFindParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((User::Table, User::Email))
            .columns([
                (Account::Table, Account::ProviderId),
                (Account::Table, Account::AccountId),
                (Account::Table, Account::PasswordHash),
                (Account::Table, Account::UserId),
            ])
            .from(Account::Table)
            .inner_join(
                User::Table,
                Expr::col((User::Table, User::Id)).eq(Expr::col((Account::Table, Account::UserId))),
            )
            .and_where(Expr::col((Account::Table, Account::ProviderId)).eq(self.provider_id))
            .and_where(Expr::col((Account::Table, Account::AccountId)).eq(self.account_id))
            .to_owned()
    }
}

impl IntoInsert for AccountCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(Account::Table)
            .columns([
                Account::ProviderId,
                Account::AccountId,
                Account::PasswordHash,
                Account::UserId,
            ])
            .values_panic([
                self.provider_id.into(),
                self.account_id.into(),
                self.password_hash.into(),
                self.user_id.to_string().into(),
            ])
            .to_owned()
    }
}
