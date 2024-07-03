mod queries;
mod repositories;

pub use repositories::{
    feeds::FeedsSqliteRepository, profiles::ProfilesSqliteRepository, users::UsersSqliteRepository,
};
use sqlx::{Error, SqlitePool};

pub async fn create_database(url: &str) -> Result<SqlitePool, Error> {
    SqlitePool::connect(url).await
}
