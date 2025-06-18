use colette_core::{
    Account,
    account::{AccountParams, AccountRepository, Error},
};
use colette_query::{IntoInsert, IntoSelect, account::AccountInsert, user::UserInsert};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder as _;

use super::{PreparedClient as _, SqliteRow};

#[derive(Debug, Clone)]
pub struct SqliteAccountRepository {
    pool: Pool,
}

impl SqliteAccountRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountRepository for SqliteAccountRepository {
    async fn query(&self, params: AccountParams) -> Result<Vec<Account>, Error> {
        let client = self.pool.get().await?;

        let accounts = client
            .interact(move |conn| {
                let (sql, values) = params.into_select().build_rusqlite(SqliteQueryBuilder);
                conn.query_prepared::<Account>(&sql, &values)
            })
            .await
            .unwrap()?;

        Ok(accounts)
    }

    async fn save(&self, data: &Account) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let data = data.to_owned();

        client
            .interact(move |conn| {
                let tx = conn.transaction()?;

                if let Some(user) = data.user {
                    let (sql, values) = UserInsert {
                        id: user.id,
                        email: &user.email,
                        display_name: user.display_name.as_deref(),
                        image_url: user.image_url.as_ref().map(|e| e.as_str()),
                        created_at: user.created_at,
                        updated_at: user.updated_at,
                    }
                    .into_insert()
                    .build_rusqlite(SqliteQueryBuilder);
                    tx.execute_prepared(&sql, &values)?;
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
                .build_rusqlite(SqliteQueryBuilder);
                tx.execute_prepared(&sql, &values)?;

                tx.commit()?;

                Ok::<_, Error>(())
            })
            .await
            .unwrap()?;

        Ok(())
    }
}

impl From<SqliteRow<'_>> for Account {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
            sub: value.get_unwrap("sub"),
            provider: value.get_unwrap("provider"),
            password_hash: value.get_unwrap("password_hash"),
            user_id: value.get_unwrap("user_id"),
            created_at: value.get_unwrap("created_at"),
            updated_at: value.get_unwrap("updated_at"),
            user: None,
        }
    }
}
