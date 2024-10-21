pub use backup::SqliteBackupRepository;
pub use bookmark::SqliteBookmarkRepository;
pub use cleanup::SqliteCleanupRepository;
pub use feed::SqliteFeedRepository;
pub use feed_entry::SqliteFeedEntryRepository;
pub use profile::SqliteProfileRepository;
pub use refresh::SqliteRefreshRepository;
pub use scraper::SqliteScraperRepository;
pub use smart_feed::SqliteSmartFeedRepository;
use sqlx::SqlitePool;
pub use tag::SqliteTagRepository;
pub use user::SqliteUserRepository;

mod backup;
mod bookmark;
mod cleanup;
mod feed;
mod feed_entry;
mod profile;
mod refresh;
mod scraper;
mod smart_feed;
mod tag;
mod user;

pub async fn migrate(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(pool).await?;

    Ok(())
}
