use chrono::{DateTime, Utc};
use colette_core::{
    Account,
    account::{AccountParams, AccountRepository, Error},
};
use colette_query::{
    IntoInsert, IntoSelect,
    account::{AccountInsert, AccountSelect},
    user::UserInsert,
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder as _;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn query(&self, params: AccountParams) -> Result<Vec<Account>, Error> {
        let (sql, values) = AccountSelect {
            id: params.id,
            sub: params.sub.as_deref(),
            provider: params.provider.as_deref(),
        }
        .into_select()
        .build_sqlx(PostgresQueryBuilder);
        let rows = sqlx::query_as_with::<_, AccountRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Account) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        if let Some(ref user) = data.user {
            let (sql, values) = UserInsert {
                id: user.id,
                email: &user.email,
                display_name: user.display_name.as_deref(),
                image_url: user.image_url.as_ref().map(|e| e.as_str()),
                created_at: user.created_at,
                updated_at: user.updated_at,
            }
            .into_insert()
            .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
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
        .build_sqlx(PostgresQueryBuilder);
        sqlx::query_with(&sql, values).execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AccountRow {
    id: Uuid,
    sub: String,
    provider: String,
    password_hash: Option<String>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<AccountRow> for Account {
    fn from(value: AccountRow) -> Self {
        Self {
            id: value.id,
            sub: value.sub,
            provider: value.provider,
            password_hash: value.password_hash,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
            user: None,
        }
    }
}
