use colette_core::{
    Account,
    accounts::{AccountCreateData, AccountFindParams, AccountRepository, Error},
    common::{Creatable, Findable},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresAccountRepository {
    pool: Pool<Postgres>,
}

impl PostgresAccountRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresAccountRepository {
    type Params = AccountFindParams;
    type Output = Result<Account, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        sqlx::query_file_as!(
            Account,
            "queries/accounts/select.sql",
            params.provider_id,
            params.account_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.account_id),
            _ => Error::Database(e),
        })
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresAccountRepository {
    type Data = AccountCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self.pool.begin().await?;

        let user_id =
            sqlx::query_file_scalar!("queries/users/insert.sql", data.email, data.display_name)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.email)
                    }
                    _ => Error::Database(e),
                })?;

        sqlx::query_file_scalar!(
            "queries/accounts/insert.sql",
            data.provider_id,
            data.account_id,
            data.password_hash,
            user_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(user_id)
    }
}

impl AccountRepository for PostgresAccountRepository {}
