pub use repositories::{
    EntriesPostgresRepository, FeedsPostgresRepository, ProfilesPostgresRepository,
    UsersPostgresRepository,
};
use sqlx::{Error, PgPool};

mod queries;
mod repositories;

pub async fn create_database(url: &str) -> Result<PgPool, Error> {
    let pool = PgPool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
