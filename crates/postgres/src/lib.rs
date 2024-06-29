use sqlx::{Error, PgPool};

pub mod queries;
pub mod repositories;

pub async fn create_database(url: &str) -> Result<PgPool, Error> {
    PgPool::connect(url).await
}
