use sqlx::{Error, PgPool};

mod queries;
mod repositories;

pub use repositories::{
    feeds::FeedsPostgresRepository, profiles::ProfilesPostgresRepository,
    users::UsersPostgresRepository,
};

pub async fn create_database(url: &str) -> Result<PgPool, Error> {
    PgPool::connect(url).await
}
