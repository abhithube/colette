pub use backup::SqliteBackupRepository;
pub use bookmark::SqliteBookmarkRepository;
pub use cleanup::SqliteCleanupRepository;
pub use feed::SqliteFeedRepository;
pub use feed_entry::SqliteFeedEntryRepository;
pub use profile::SqliteProfileRepository;
pub use smart_feed::SqliteSmartFeedRepository;
pub use tag::SqliteTagRepository;
pub use user::SqliteUserRepository;

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
mod smart_feed;
mod tag;
mod user;

pub async fn migrate(pool: &mut deadpool_sqlite::Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.get().await?;

    conn.interact(move |conn| migrations::runner().run(conn))
        .await??;

    Ok(())
}
