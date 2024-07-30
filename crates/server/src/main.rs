use std::{error::Error, str::FromStr, sync::Arc};

use app::App;
use chrono::Local;
use colette_backup::OpmlManager;
use colette_core::{
    auth::AuthService, bookmarks::BookmarksService, entries::EntriesService, feeds::FeedsService,
    profiles::ProfilesService, tags::TagsService, utils::task::Task,
};
use colette_password::Argon2Hasher;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_repositories::{
    BookmarksSqlRepository, EntriesSqlRepository, FeedsSqlRepository, ProfilesSqlRepository,
    TagsSqlRepository, UsersSqlRepository,
};
use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
use colette_tasks::{CleanupTask, RefreshTask};
use cron::Schedule;
use migrations::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend};
use tokio::time;
use tower_sessions::{
    session::{Id, Record},
    session_store::ExpiredDeletion,
    SessionStore,
};
use tower_sessions_sqlx_store::{PostgresStore, SqliteStore};

mod app;
mod auth;
mod bookmarks;
mod common;
mod entries;
mod feeds;
mod profiles;
mod tags;

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = colette_config::load_config()?;

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(100).min_connections(5);
    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;

    let bookmarks_repository = Arc::new(BookmarksSqlRepository::new(db.clone()));
    let entries_repository = Arc::new(EntriesSqlRepository::new(db.clone()));
    let feeds_repository = Arc::new(FeedsSqlRepository::new(db.clone()));
    let profiles_repository = Arc::new(ProfilesSqlRepository::new(db.clone()));
    let tags_repository = Arc::new(TagsSqlRepository::new(db.clone()));
    let users_repository = Arc::new(UsersSqlRepository::new(db.clone()));

    let (session_database, deletion_task) = match db.get_database_backend() {
        DatabaseBackend::Postgres => {
            let store = PostgresStore::new(db.get_postgres_connection_pool().clone());
            store.migrate().await?;

            let deletion_task = tokio::task::spawn(
                store
                    .clone()
                    .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
            );

            (SessionDatabase::Postgres(store), deletion_task)
        }
        DatabaseBackend::Sqlite => {
            let store = SqliteStore::new(db.get_sqlite_connection_pool().clone());
            store.migrate().await?;

            let deletion_task = tokio::task::spawn(
                store
                    .clone()
                    .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
            );

            (SessionDatabase::Sqlite(store), deletion_task)
        }
        _ => panic!("Only PostgreSQL and SQLite supported"),
    };

    let feed_scraper = Arc::new(DefaultFeedScraper::new(register_feed_plugins()));

    if config.refresh_enabled {
        let feed_scraper = feed_scraper.clone();
        let feeds_repository = feeds_repository.clone();
        let profiles_repository = profiles_repository.clone();

        let schedule = Schedule::from_str(&config.cron_refresh).unwrap();

        tokio::spawn(async move {
            let refresh_task = RefreshTask::new(
                feed_scraper.clone(),
                feeds_repository.clone(),
                profiles_repository.clone(),
            );

            loop {
                let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
                let duration = (upcoming - Local::now()).to_std().unwrap();

                time::sleep(duration).await;

                let start = Local::now();
                println!("Started refresh task at: {}", start);

                match refresh_task.run().await {
                    Ok(_) => {
                        let elasped = (Local::now().time() - start.time()).num_milliseconds();
                        println!("Finished refresh task in {} ms", elasped);
                    }
                    Err(e) => {
                        println!("Failed refresh task: {}", e);
                    }
                }
            }
        });
    }

    {
        let feeds_repository = feeds_repository.clone();

        let schedule = Schedule::from_str(CRON_CLEANUP).unwrap();

        tokio::spawn(async move {
            let cleanup_task = CleanupTask::new(feeds_repository.clone());

            loop {
                let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
                let duration = (upcoming - Local::now()).to_std().unwrap();

                time::sleep(duration).await;

                let start = Local::now();
                println!("Started cleanup task at: {}", start);

                match cleanup_task.run().await {
                    Ok(_) => {
                        let elasped = (Local::now().time() - start.time()).num_milliseconds();
                        println!("Finished cleanup task in {} ms", elasped);
                    }
                    Err(e) => {
                        println!("Failed cleanup task: {}", e);
                    }
                }
            }
        });
    }

    let state = common::Context {
        auth_service: AuthService::new(
            users_repository,
            profiles_repository.clone(),
            Arc::new(Argon2Hasher {}),
        )
        .into(),
        bookmark_service: BookmarksService::new(
            bookmarks_repository,
            Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
        )
        .into(),
        entries_service: EntriesService::new(entries_repository).into(),
        feeds_service: FeedsService::new(feeds_repository, feed_scraper, Arc::new(OpmlManager))
            .into(),
        profiles_service: ProfilesService::new(profiles_repository).into(),
        tags_service: TagsService::new(tags_repository).into(),
    };

    App::new(state, config, session_database).start().await?;

    deletion_task.await??;

    Ok(())
}

#[derive(Clone, Debug)]
pub enum SessionDatabase {
    Postgres(PostgresStore),
    Sqlite(SqliteStore),
}

#[async_trait::async_trait]
impl SessionStore for SessionDatabase {
    async fn create(
        &self,
        session_record: &mut Record,
    ) -> Result<(), tower_sessions::session_store::Error> {
        match self {
            SessionDatabase::Postgres(store) => store.create(session_record).await,
            SessionDatabase::Sqlite(store) => store.create(session_record).await,
        }
    }

    async fn save(
        &self,
        session_record: &Record,
    ) -> Result<(), tower_sessions::session_store::Error> {
        match self {
            SessionDatabase::Postgres(store) => store.save(session_record).await,
            SessionDatabase::Sqlite(store) => store.save(session_record).await,
        }
    }

    async fn load(
        &self,
        session_id: &Id,
    ) -> Result<Option<Record>, tower_sessions::session_store::Error> {
        match self {
            SessionDatabase::Postgres(store) => store.load(session_id).await,
            SessionDatabase::Sqlite(store) => store.load(session_id).await,
        }
    }

    async fn delete(&self, session_id: &Id) -> Result<(), tower_sessions::session_store::Error> {
        match self {
            SessionDatabase::Postgres(store) => store.delete(session_id).await,
            SessionDatabase::Sqlite(store) => store.delete(session_id).await,
        }
    }
}
