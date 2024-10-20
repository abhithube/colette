pub use backup::SqliteBackupRepository;
pub use bookmark::SqliteBookmarkRepository;
pub use cleanup::SqliteCleanupRepository;
pub use feed::SqliteFeedRepository;
pub use feed_entry::SqliteFeedEntryRepository;
use futures::{future::BoxFuture, FutureExt};
pub use profile::SqliteProfileRepository;
pub use refresh::SqliteRefreshRepository;
pub use smart_feed::SqliteSmartFeedRepository;
use sqlx::{
    error::BoxDynError,
    migrate::{Migration, MigrationSource, Migrator},
    SqlitePool,
};
pub use tag::SqliteTagRepository;
pub use user::SqliteUserRepository;

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

pub async fn migrate(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let migrator = Migrator::new(MigrationList(migration::migrations())).await?;

    migrator.run(pool).await?;

    Ok(())
}
