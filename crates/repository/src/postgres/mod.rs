pub use account::PostgresAccountRepository;
pub use api_key::PostgresApiKeyRepository;
pub use bookmark::PostgresBookmarkRepository;
pub use collection::PostgresCollectionRepository;
use deadpool_postgres::{Object, Transaction};
pub use feed::PostgresFeedRepository;
pub use feed_entry::PostgresFeedEntryRepository;
pub use job::PostgresJobRepository;
use sea_query_postgres::PostgresValues;
pub use stream::PostgresStreamRepository;
pub use subscription::PostgresSubscriptionRepository;
pub use subscription_entry::PostgresSubscriptionEntryRepository;
pub use tag::PostgresTagRepository;
use tokio_postgres::Row;
pub use user::PostgresUserRepository;
use uuid::Uuid;

mod account;
mod api_key;
mod bookmark;
mod collection;
mod feed;
mod feed_entry;
mod job;
mod stream;
mod subscription;
mod subscription_entry;
mod tag;
mod user;

struct PgRow<'a>(&'a Row);

struct IdRow {
    id: Uuid,
}

impl From<PgRow<'_>> for IdRow {
    fn from(PgRow(value): PgRow<'_>) -> Self {
        Self {
            id: value.get("id"),
        }
    }
}

#[async_trait::async_trait]
trait PreparedClient {
    async fn query_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<Vec<T>, tokio_postgres::Error>;

    async fn query_opt_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<Option<T>, tokio_postgres::Error>;

    async fn query_one_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<T, tokio_postgres::Error>;

    async fn execute_prepared(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<u64, tokio_postgres::Error>;
}

#[async_trait::async_trait]
impl PreparedClient for Object {
    async fn query_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<Vec<T>, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        let rows = self.query(&stmt, &values.as_params()).await?;

        Ok(rows.into_iter().map(|e| PgRow(&e).into()).collect())
    }

    async fn query_opt_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<Option<T>, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        let row = self.query_opt(&stmt, &values.as_params()).await?;

        Ok(row.map(|e| PgRow(&e).into()))
    }

    async fn query_one_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<T, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        let row = self.query_one(&stmt, &values.as_params()).await?;

        Ok(PgRow(&row).into())
    }

    async fn execute_prepared(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<u64, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        self.execute(&stmt, &values.as_params()).await
    }
}

#[async_trait::async_trait]
impl PreparedClient for Transaction<'_> {
    async fn query_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<Vec<T>, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        let rows = self.query(&stmt, &values.as_params()).await?;

        Ok(rows.into_iter().map(|e| PgRow(&e).into()).collect())
    }

    async fn query_opt_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<Option<T>, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        let row = self.query_opt(&stmt, &values.as_params()).await?;

        Ok(row.map(|e| PgRow(&e).into()))
    }

    async fn query_one_prepared<T: for<'a> From<PgRow<'a>>>(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<T, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        let row = self.query_one(&stmt, &values.as_params()).await?;

        Ok(PgRow(&row).into())
    }

    async fn execute_prepared(
        &self,
        sql: &str,
        values: &PostgresValues,
    ) -> Result<u64, tokio_postgres::Error> {
        let stmt = self.prepare_cached(sql).await?;
        self.execute(&stmt, &values.as_params()).await
    }
}
