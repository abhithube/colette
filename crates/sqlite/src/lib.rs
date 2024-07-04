mod queries;
mod repositories;

pub use repositories::{
    feeds::FeedsSqliteRepository, profiles::ProfilesSqliteRepository, users::UsersSqliteRepository,
};
use sqlx::{Error, SqlitePool};

pub async fn create_database(url: &str) -> Result<SqlitePool, Error> {
    let pool = SqlitePool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
