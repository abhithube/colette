pub use queries::{feeds::iterate as iterate_feeds, profiles::iterate as iterate_profiles};
pub use repositories::{
    EntriesSqliteRepository, FeedsSqliteRepository, ProfilesSqliteRepository, UsersSqliteRepository,
};
use sqlx::{Error, SqlitePool};

mod queries;
mod repositories;

pub type Pool = SqlitePool;

pub async fn create_database(url: &str) -> Result<SqlitePool, Error> {
    let pool = SqlitePool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
