use sqlx::PgPool;

mod bookmarks;
mod entries;
mod feeds;
mod profiles;
mod tags;
mod users;

pub struct PostgresRepository {
    pub(crate) pool: PgPool,
}

impl PostgresRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub async fn initialize(url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPool::connect(url).await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    Ok(pool)
}
