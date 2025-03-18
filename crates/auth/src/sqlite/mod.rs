use sqlx::{Pool, Sqlite};

mod password;
mod session;
mod user;

#[derive(Clone)]
pub struct SqliteBackend {
    pool: Pool<Sqlite>,
}

impl SqliteBackend {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}
