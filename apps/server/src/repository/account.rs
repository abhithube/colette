use colette_core::{
    Account,
    account::{AccountCreateParams, AccountFindParams, AccountRepository, Error},
    common::Transaction,
};
use colette_model::{AccountRow, accounts, users};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult,
    sea_query::{Expr, Query},
};

#[derive(Clone)]
pub struct SqliteAccountRepository {
    db: DatabaseConnection,
}

impl SqliteAccountRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_account(&self, params: AccountFindParams) -> Result<Account, Error> {
        let query = Query::select()
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
                Expr::col((accounts::Entity, accounts::Column::ProviderId)).eq(params.provider_id),
            )
            .and_where(
                Expr::col((accounts::Entity, accounts::Column::AccountId))
                    .eq(params.account_id.as_str()),
            )
            .to_owned();

        let Some(account) =
            AccountRow::find_by_statement(self.db.get_database_backend().build(&query))
                .one(&self.db)
                .await?
        else {
            return Err(Error::NotFound(params.account_id));
        };

        Ok(account.into())
    }

    async fn create_account(
        &self,
        tx: &dyn Transaction,
        params: AccountCreateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::insert()
            .into_table(accounts::Entity)
            .columns([
                accounts::Column::ProviderId,
                accounts::Column::AccountId,
                accounts::Column::PasswordHash,
                accounts::Column::UserId,
            ])
            .values_panic([
                params.provider_id.into(),
                params.account_id.into(),
                params.password_hash.into(),
                params.user_id.to_string().into(),
            ])
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }
}
