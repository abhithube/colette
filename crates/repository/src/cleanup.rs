use colette_core::cleanup::{CleanupRepository, Error};
use sea_orm::DatabaseConnection;

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
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut count = colette_postgres::feed_entry::delete_many(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned feed entries", count);
        }

        count = colette_postgres::feed::delete_many(&mut *tx)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned feeds", count);
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn cleanup_tags(&self) -> Result<(), Error> {
        let count = colette_postgres::tag::delete_many(self.db.get_postgres_connection_pool())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count > 0 {
            println!("Deleted {} orphaned tags", count);
        }

        Ok(())
    }
}
