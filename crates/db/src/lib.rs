use sea_orm::{DatabaseConnection, SqlxPostgresConnector};
use sqlx::PgPool;

mod bookmarks;
mod entries;
mod feeds;
mod profiles;
mod tags;
mod users;

pub struct PostgresRepository {
    pub(crate) pool: PgPool,
    pub(crate) db: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: pool.clone(),
            db: SqlxPostgresConnector::from_sqlx_postgres_pool(pool),
        }
    }
}

pub async fn initialize(url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(url).await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    Ok(pool)
}
