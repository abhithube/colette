use deadpool_postgres::Pool;

mod password;
mod session;
mod user;

#[derive(Clone)]
pub struct PostgresBackend {
    pool: Pool,
}

impl PostgresBackend {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}
