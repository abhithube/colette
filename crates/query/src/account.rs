use colette_core::account::{AccountCreateParams, AccountFindParams};
use colette_model::{accounts, users};
use sea_query::{Expr, InsertStatement, Query, SelectStatement};

use crate::{IntoInsert, IntoSelect};

impl IntoSelect for AccountFindParams {
    fn into_select(self) -> SelectStatement {
        Query::select()
            .column((users::Entity, users::Column::Email))
            .columns([
                (accounts::Entity, accounts::Column::ProviderId),
                (accounts::Entity, accounts::Column::AccountId),
                (accounts::Entity, accounts::Column::PasswordHash),
                (accounts::Entity, accounts::Column::UserId),
            ])
            .from(accounts::Entity)
            .inner_join(
                users::Entity,
                Expr::col((users::Entity, users::Column::Id))
                    .eq(Expr::col((accounts::Entity, accounts::Column::UserId))),
            )
            .and_where(
                Expr::col((accounts::Entity, accounts::Column::ProviderId)).eq(self.provider_id),
            )
            .and_where(
                Expr::col((accounts::Entity, accounts::Column::AccountId)).eq(self.account_id),
            )
            .to_owned()
    }
}

impl IntoInsert for AccountCreateParams {
    fn into_insert(self) -> InsertStatement {
        Query::insert()
            .into_table(accounts::Entity)
            .columns([
                accounts::Column::ProviderId,
                accounts::Column::AccountId,
                accounts::Column::PasswordHash,
                accounts::Column::UserId,
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
