use sqlx::PgPool;

mod bookmarks;
mod common;
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
