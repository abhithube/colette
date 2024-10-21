pub use backup::PostgresBackupRepository;
pub use bookmark::PostgresBookmarkRepository;
pub use cleanup::PostgresCleanupRepository;
pub use feed::PostgresFeedRepository;
pub use feed_entry::PostgresFeedEntryRepository;
use futures::{future::BoxFuture, FutureExt};
pub use profile::PostgresProfileRepository;
pub use refresh::PostgresRefreshRepository;
pub use scraper::PostgresScraperRepository;
pub use smart_feed::PostgresSmartFeedRepository;
use sqlx::{
    error::BoxDynError,
    migrate::{Migration, MigrationSource, Migrator},
    PgPool,
};
pub use tag::PostgresTagRepository;
pub use user::PostgresUserRepository;

mod backup;
mod bookmark;
mod cleanup;
mod feed;
mod feed_entry;
#[allow(dead_code)]
#[allow(non_snake_case)]
mod migration;
mod profile;
mod refresh;
mod scraper;
mod smart_feed;
mod tag;
mod user;

#[derive(Debug)]
struct MigrationList(Vec<Migration>);

impl MigrationSource<'static> for MigrationList {
    fn resolve(self) -> BoxFuture<'static, Result<Vec<Migration>, BoxDynError>> {
        async { Ok(self.0) }.boxed()
    }
}

pub async fn migrate(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let migrator = Migrator::new(MigrationList(migration::migrations())).await?;

    migrator.run(pool).await?;

    Ok(())
}
