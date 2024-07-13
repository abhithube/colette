pub use queries::{feeds::iterate as iterate_feeds, profiles::iterate as iterate_profiles};
pub use repositories::{
    CollectionsPostgresRepository, EntriesPostgresRepository, FeedsPostgresRepository,
    ProfilesPostgresRepository, UsersPostgresRepository,
};
use sqlx::{Error, PgPool};

mod queries;
mod repositories;

pub type Pool = PgPool;

pub async fn create_database(url: &str) -> Result<PgPool, Error> {
    let pool = PgPool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
