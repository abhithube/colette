use std::any::Any;

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
