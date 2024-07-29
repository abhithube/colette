pub use sea_orm_migration::prelude::*;

mod m0001_initial_feed;
mod m0002_initial_user;
mod m0003_initial_profile_feed;
mod m0004_initial_collection;
mod m0005_initial_tag;
mod postgres;
mod sqlite;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m0001_initial_feed::Migration),
            Box::new(m0002_initial_user::Migration),
            Box::new(m0003_initial_profile_feed::Migration),
            Box::new(m0004_initial_collection::Migration),
            Box::new(m0005_initial_tag::Migration),
        ]
    }
}
