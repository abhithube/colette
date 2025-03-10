use std::any::Any;

use chrono::{DateTime, Utc};
use colette_core::common::{Transaction, TransactionManager};
use futures::lock::Mutex;
use sqlx::{Pool, Sqlite};

#[derive(Debug)]
pub struct SqliteTransaction {
    tx: Mutex<sqlx::Transaction<'static, Sqlite>>,
}

#[async_trait::async_trait]
impl Transaction for SqliteTransaction {
    async fn commit(self: Box<Self>) -> Result<(), sqlx::Error> {
        self.tx.into_inner().commit().await
    }

    async fn rollback(self: Box<Self>) -> Result<(), sqlx::Error> {
        self.tx.into_inner().rollback().await
    }

    fn as_any(&self) -> &dyn Any {
        &self.tx
    }
}

// impl SqliteTransaction {
//     pub async fn execute<'a, E>(
//         &self,
//         query: E,
//     ) -> Result<sqlx::sqlite::SqliteQueryResult, sqlx::Error>
//     where
//         E: sqlx::Execute<'a, Sqlite> + Send,
//     {
//         let mut guard = self.tx.lock().unwrap();
//         let tx = guard.as_mut();
//         Ok(query.execute(&mut *tx).await?)
//     }
// }

#[derive(Debug, Clone)]
pub struct SqliteTransactionManager {
    pool: Pool<Sqlite>,
}

impl SqliteTransactionManager {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TransactionManager for SqliteTransactionManager {
    async fn begin(&self) -> Result<Box<dyn Transaction>, sqlx::Error> {
        let tx = self.pool.begin().await?;

        Ok(Box::new(SqliteTransaction { tx: Mutex::new(tx) }))
    }
}

pub(crate) fn parse_timestamp(value: i32) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(value.into(), 0)
}
