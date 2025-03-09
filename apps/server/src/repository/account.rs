use colette_core::{
    Account,
    account::{AccountCreateParams, AccountFindParams, AccountRepository, Error},
    common::Transaction,
};
use colette_model::AccountRow;
use colette_query::{IntoInsert, IntoSelect};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult};

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
        let account_id = params.account_id.clone();

        let Some(account) = AccountRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .one(&self.db)
        .await?
        else {
            return Err(Error::NotFound(account_id));
        };

        Ok(account.into())
    }

    async fn create_account(
        &self,
        tx: &dyn Transaction,
        params: AccountCreateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_insert()))
            .await?;

        Ok(())
    }
}
