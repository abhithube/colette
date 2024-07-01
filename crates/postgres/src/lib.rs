use sqlx::{Error, PgPool};

mod queries;
mod repositories;

pub use repositories::{profiles::ProfilesPostgresRepository, users::UsersPostgresRepository};

pub async fn create_database(url: &str) -> Result<PgPool, Error> {
    PgPool::connect(url).await
}
