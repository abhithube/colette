#[allow(unused_imports)]
use sea_orm::DatabaseBackend;
use sea_orm_migration::prelude::*;

#[cfg(feature = "postgres")]
use crate::postgres;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        match manager.get_database_backend() {
            #[cfg(feature = "postgres")]
            DatabaseBackend::Postgres => {
                postgres::create_updated_at_fn(manager).await?;
            }
            _ => {}
        }

        Ok(())
    }
}
