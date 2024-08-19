use sea_orm::DatabaseConnection;

mod bookmark;
mod collection;
mod feed;
mod feed_entry;
mod folder;
mod profile;
mod queries;
mod tag;
mod user;

pub struct SqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
