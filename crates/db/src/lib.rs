use colette_migrations::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection, DbErr};

mod bookmarks;
mod entries;
mod feeds;
mod profiles;
mod tags;
mod users;

pub struct PostgresRepository {
    pub(crate) db: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

pub async fn initialize(url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(url).await?;

    Migrator::up(&db, None).await?;

    Ok(db)
}
