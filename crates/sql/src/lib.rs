use sea_orm::DatabaseConnection;

mod bookmarks;
mod collections;
mod feed_entries;
mod feeds;
mod profiles;
mod tags;
mod users;

pub struct SqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl SqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
