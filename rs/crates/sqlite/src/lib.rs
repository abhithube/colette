pub use repositories::{
    BookmarksSqliteRepository, CollectionsSqliteRepository, EntriesSqliteRepository,
    FeedsSqliteRepository, ProfilesSqliteRepository, UsersSqliteRepository,
};

mod queries;
mod repositories;

pub async fn create_database(url: &str) -> Result<sqlx::SqlitePool, sqlx::Error> {
    let pool = sqlx::SqlitePool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
