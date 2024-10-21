pub use backup::PostgresBackupRepository;
pub use bookmark::PostgresBookmarkRepository;
pub use cleanup::PostgresCleanupRepository;
pub use feed::PostgresFeedRepository;
pub use feed_entry::PostgresFeedEntryRepository;
pub use profile::PostgresProfileRepository;
pub use refresh::PostgresRefreshRepository;
pub use scraper::PostgresScraperRepository;
pub use smart_feed::PostgresSmartFeedRepository;
use sqlx::PgPool;
pub use tag::PostgresTagRepository;
pub use user::PostgresUserRepository;

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

pub async fn migrate(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::migrate!("./migrations").run(pool).await?;

    Ok(())
}
