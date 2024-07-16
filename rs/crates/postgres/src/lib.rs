pub use repositories::{
    BookmarksPostgresRepository, CollectionsPostgresRepository, EntriesPostgresRepository,
    FeedsPostgresRepository, ProfilesPostgresRepository, UsersPostgresRepository,
};

mod queries;
mod repositories;

pub async fn create_database(url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    let pool = sqlx::PgPool::connect(url).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
