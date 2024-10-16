use std::ops::DerefMut;

pub use backup::PostgresBackupRepository;
pub use bookmark::PostgresBookmarkRepository;
pub use cleanup::PostgresCleanupRepository;
pub use feed::PostgresFeedRepository;
pub use feed_entry::PostgresFeedEntryRepository;
pub use profile::PostgresProfileRepository;
pub use refresh::PostgresRefreshRepository;
pub use session::PostgresSessionRepository;
pub use smart_feed::PostgresSmartFeedRepository;
pub use tag::PostgresTagRepository;
pub use user::PostgresUserRepository;

refinery::embed_migrations!("src/migration");

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
mod session;
mod smart_feed;
mod tag;
mod user;

pub async fn migrate(pool: &mut deadpool_postgres::Pool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get().await?;
    let client = conn.deref_mut().deref_mut();

    migrations::runner().run_async(client).await?;

    Ok(())
}
