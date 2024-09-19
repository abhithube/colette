use colette_core::cleanup::{CleanupRepository, Error};
use sea_orm::{DatabaseConnection, DbErr, TransactionTrait};

use crate::query;

pub struct CleanupSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl CleanupSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl CleanupRepository for CleanupSqlRepository {
    async fn cleanup_feeds(&self) -> Result<(), Error> {
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let result = query::feed_entry::delete_many(txn).await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feed entries", result.rows_affected);
                    }

                    let result = query::feed::delete_many(txn).await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned feeds", result.rows_affected);
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_tags(&self) -> Result<(), Error> {
        self.db
            .transaction::<_, (), DbErr>(|txn| {
                Box::pin(async move {
                    let result = query::tag::delete_many(txn).await?;
                    if result.rows_affected > 0 {
                        println!("Deleted {} orphaned tags", result.rows_affected);
                    }

                    Ok(())
                })
            })
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}
