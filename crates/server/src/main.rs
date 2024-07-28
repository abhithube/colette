#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("features \"postgres\" and \"sqlite\" are mutually exclusive");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Either feature \"postgres\" or \"sqlite\" must be enabled");

use std::{error::Error, str::FromStr, sync::Arc};

use app::App;
use chrono::Local;
use colette_backup::OpmlManager;
use colette_core::{
    auth::AuthService, bookmarks::BookmarksService, collections::CollectionsService,
    entries::EntriesService, feeds::FeedsService, profiles::ProfilesService, tags::TagsService,
    utils::task::Task,
};
use colette_password::Argon2Hasher;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_postgres::{
    BookmarksSqlRepository, CollectionsSqlRepository, EntriesSqlRepository, FeedsSqlRepository,
    ProfilesSqlRepository, TagsSqlRepository, UsersSqlRepository,
};
use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
use colette_tasks::{CleanupTask, RefreshTask};
use cron::Schedule;
use migrations::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database};
use tokio::time;
use tower_sessions::session_store::ExpiredDeletion;
#[cfg(feature = "postgres")]
use tower_sessions_sqlx_store::PostgresStore;
#[cfg(feature = "sqlite")]
use tower_sessions_sqlx_store::SqliteStore;

mod app;
mod auth;
mod bookmarks;
mod collections;
mod common;
mod entries;
mod feeds;
mod profiles;
mod tags;

const CRON_CLEANUP: &str = "0 0 * * * *";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = colette_config::load_config()?;

    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(100).min_connections(5);
    let db = Database::connect(opt).await?;
    Migrator::up(&db, None).await?;

    let bookmarks_repository = Arc::new(BookmarksSqlRepository::new(db.clone()));
    let collections_repository = Arc::new(CollectionsSqlRepository::new(db.clone()));
    let entries_repository = Arc::new(EntriesSqlRepository::new(db.clone()));
    let feeds_repository = Arc::new(FeedsSqlRepository::new(db.clone()));
    let profiles_repository = Arc::new(ProfilesSqlRepository::new(db.clone()));
    let tags_repository = Arc::new(TagsSqlRepository::new(db.clone()));
    let users_repository = Arc::new(UsersSqlRepository::new(db.clone()));

    let (session_store, cleanup) = {
        #[cfg(feature = "postgres")]
        let store = PostgresStore::new(db.get_postgres_connection_pool().clone());
        #[cfg(feature = "sqlite")]
        let store = SqliteStore::new(db.get_sqlite_connection_pool().clone());

        let deletion_task = {
            store.migrate().await?;

            tokio::task::spawn(
                store
                    .clone()
                    .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
            )
        };

        (store, deletion_task)
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
        collections_service: CollectionsService::new(collections_repository).into(),
        entries_service: EntriesService::new(entries_repository).into(),
        feeds_service: FeedsService::new(feeds_repository, feed_scraper, Arc::new(OpmlManager))
            .into(),
        profiles_service: ProfilesService::new(profiles_repository).into(),
        tags_service: TagsService::new(tags_repository).into(),
    };

    App::new(state, config, session_store).start().await?;

    cleanup.await??;

    Ok(())
}
