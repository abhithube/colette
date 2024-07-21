pub use repositories::{
    BookmarksPostgresRepository, CollectionsPostgresRepository, EntriesPostgresRepository,
    FeedsPostgresRepository, ProfilesPostgresRepository, UsersPostgresRepository,
};
pub use sqlx::PgPool;

mod queries;
mod repositories;

pub async fn create_database(url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = sqlx::PgPool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
