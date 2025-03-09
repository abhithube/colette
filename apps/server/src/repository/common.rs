use std::any::Any;

use colette_core::common::{Transaction, TransactionManager};
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

#[derive(Debug)]
pub struct SqliteTransaction {
    tx: DatabaseTransaction,
}

#[async_trait::async_trait]
impl Transaction for SqliteTransaction {
    async fn commit(self: Box<Self>) -> Result<(), DbErr> {
        self.tx.commit().await
    }

    async fn rollback(self: Box<Self>) -> Result<(), DbErr> {
        self.tx.rollback().await
    }

    fn as_any(&self) -> &dyn Any {
        &self.tx
    }
}

#[derive(Debug, Clone)]
pub struct SqliteTransactionManager {
    db: DatabaseConnection,
}

impl SqliteTransactionManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TransactionManager for SqliteTransactionManager {
    async fn begin(&self) -> Result<Box<dyn Transaction>, DbErr> {
        let tx = self.db.begin().await?;

        Ok(Box::new(SqliteTransaction { tx }))
    }
}
