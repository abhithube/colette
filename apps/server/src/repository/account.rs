use colette_core::{
    Account,
    account::{AccountCreateParams, AccountFindParams, AccountRepository, Error},
    common::Transaction,
};
use colette_query::{IntoInsert, IntoSelect};
use futures::lock::Mutex;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct SqliteAccountRepository {
    pool: Pool<Sqlite>,
}

impl SqliteAccountRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn find_account(&self, params: AccountFindParams) -> Result<Account, Error> {
        let account_id = params.account_id.clone();

        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let row = sqlx::query_as_with::<_, AccountRow, _>(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(account_id),
                _ => Error::Database(e),
            })?;

        Ok(row.into())
    }

    async fn create_account(
        &self,
        tx: &dyn Transaction,
        params: AccountCreateParams,
    ) -> Result<(), Error> {
        let mut tx = tx
            .as_any()
            .downcast_ref::<Mutex<sqlx::Transaction<'static, Sqlite>>>()
            .unwrap()
            .lock()
            .await;

        let (sql, values) = params.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(tx.as_mut()).await?;

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    email: String,
    provider_id: String,
    account_id: String,
    password_hash: Option<String>,
    user_id: String,
}

impl From<AccountRow> for Account {
    fn from(value: AccountRow) -> Self {
        Self {
            email: value.email,
            provider_id: value.provider_id,
            account_id: value.account_id,
            password_hash: value.password_hash,
            id: value.user_id.parse().unwrap(),
        }
    }
}
