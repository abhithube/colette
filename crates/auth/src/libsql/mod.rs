use libsql::Connection;

mod password;
mod session;
mod user;

#[derive(Clone)]
pub struct LibsqlBackend {
    conn: Connection,
}

impl LibsqlBackend {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}
