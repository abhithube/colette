use colette_core::{
    Account,
    account::{AccountParams, AccountRepository, Error},
};
use colette_query::{IntoInsert, IntoSelect, account::AccountInsert, user::UserInsert};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;

use super::{PgRow, PreparedClient as _};

#[derive(Debug, Clone)]
pub struct PostgresAccountRepository {
    pool: Pool,
}

impl PostgresAccountRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn query(&self, params: AccountParams) -> Result<Vec<Account>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);
        let accounts = client.query_prepared::<Account>(&sql, &values).await?;

        Ok(accounts)
    }

    async fn save(&self, data: &Account) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        if let Some(user) = &data.user {
            let (sql, values) = UserInsert {
                id: user.id,
                email: &user.email,
                display_name: user.display_name.as_deref(),
                image_url: user.image_url.as_ref().map(|e| e.as_str()),
                created_at: user.created_at,
                updated_at: user.updated_at,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);

            tx.execute_prepared(&sql, &values).await?;
        }

        let (sql, values) = AccountInsert {
            id: data.id,
            sub: &data.sub,
            provider: &data.provider,
            password_hash: data.password_hash.as_deref(),
            user_id: data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
        }
        .into_insert()
        .build_postgres(PostgresQueryBuilder);

        tx.execute_prepared(&sql, &values).await?;

        tx.commit().await?;

        Ok(())
    }
}

impl From<PgRow<'_>> for Account {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            sub: value.get("sub"),
            provider: value.get("provider"),
            password_hash: value.get("password_hash"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            user: None,
        }
    }
}
