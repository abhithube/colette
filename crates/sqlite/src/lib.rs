pub use repositories::{
    EntriesSqliteRepository, FeedsSqliteRepository, ProfilesSqliteRepository, UsersSqliteRepository,
};
use sqlx::{Error, SqlitePool};

mod queries;
mod repositories;

pub async fn create_database(url: &str) -> Result<SqlitePool, Error> {
    let pool = SqlitePool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
