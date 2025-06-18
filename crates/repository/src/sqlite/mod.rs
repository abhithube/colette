pub use account::SqliteAccountRepository;
pub use api_key::SqliteApiKeyRepository;
pub use bookmark::SqliteBookmarkRepository;
pub use collection::SqliteCollectionRepository;
pub use feed::SqliteFeedRepository;
pub use feed_entry::SqliteFeedEntryRepository;
pub use job::SqliteJobRepository;
use rusqlite::{Connection, OptionalExtension, Row, Transaction};
use sea_query_rusqlite::RusqliteValues;
pub use stream::SqliteStreamRepository;
pub use subscription::SqliteSubscriptionRepository;
pub use subscription_entry::SqliteSubscriptionEntryRepository;
pub use tag::SqliteTagRepository;
pub use user::SqliteUserRepository;
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

struct SqliteRow<'a>(&'a Row<'a>);

struct IdRow {
    id: Uuid,
}

impl From<SqliteRow<'_>> for IdRow {
    fn from(SqliteRow(value): SqliteRow<'_>) -> Self {
        Self {
            id: value.get_unwrap("id"),
        }
    }
}

trait PreparedClient {
    fn query_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<Vec<T>, rusqlite::Error>;

    fn query_opt_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<Option<T>, rusqlite::Error>;

    fn query_one_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<T, rusqlite::Error>;

    fn execute_prepared(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<usize, rusqlite::Error>;
}

impl PreparedClient for Connection {
    fn query_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<Vec<T>, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        let mut mapped = Vec::new();
        while let Some(row) = rows.next()? {
            mapped.push(SqliteRow(row).into());
        }

        Ok(mapped)
    }

    fn query_opt_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<Option<T>, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        stmt.query_row(&*values.as_params(), |e| Ok(SqliteRow(e).into()))
            .optional()
    }

    fn query_one_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<T, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        stmt.query_row(&*values.as_params(), |e| Ok(SqliteRow(e).into()))
    }

    fn execute_prepared(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<usize, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        stmt.execute(&*values.as_params())
    }
}

impl PreparedClient for Transaction<'_> {
    fn query_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<Vec<T>, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        let mut rows = stmt.query(&*values.as_params())?;

        let mut mapped = Vec::new();
        while let Some(row) = rows.next()? {
            mapped.push(SqliteRow(row).into());
        }

        Ok(mapped)
    }

    fn query_opt_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<Option<T>, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        stmt.query_row(&*values.as_params(), |e| Ok(SqliteRow(e).into()))
            .optional()
    }

    fn query_one_prepared<T: for<'a> From<SqliteRow<'a>>>(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<T, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        stmt.query_row(&*values.as_params(), |e| Ok(SqliteRow(e).into()))
    }

    fn execute_prepared(
        &self,
        sql: &str,
        values: &RusqliteValues,
    ) -> Result<usize, rusqlite::Error> {
        let mut stmt = self.prepare_cached(sql)?;
        stmt.execute(&*values.as_params())
    }
}
