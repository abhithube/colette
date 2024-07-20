#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("features \"postgres\" and \"sqlite\" are mutually exclusive");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Either feature \"postgres\" or \"sqlite\" must be enabled");

use std::{error::Error, str::FromStr, sync::Arc};

use app::App;
use colette_core::{
    auth::AuthService, bookmarks::BookmarksService, collections::CollectionsService,
    entries::EntriesService, feeds::FeedsService, profiles::ProfilesService, utils::task::Task,
};
use colette_password::Argon2Hasher;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
#[cfg(feature = "postgres")]
use colette_postgres::{
    BookmarksPostgresRepository, CollectionsPostgresRepository, EntriesPostgresRepository,
    FeedsPostgresRepository, ProfilesPostgresRepository, UsersPostgresRepository,
};
use colette_scraper::{
    BookmarkScraper, DefaultBookmarkExtractor, DefaultBookmarkPostprocessor, DefaultDownloader,
    DefaultFeedExtractor, DefaultFeedPostprocessor, FeedScraper,
};
#[cfg(feature = "sqlite")]
use colette_sqlite::{
    BookmarksSqliteRepository, CollectionsSqliteRepository, EntriesSqliteRepository,
    FeedsSqliteRepository, ProfilesSqliteRepository, UsersSqliteRepository,
};
use colette_tasks::{CleanupTask, RefreshTask};
use cron::Schedule;
use tokio_cron_scheduler::{Job, JobScheduler};
#[cfg(not(feature = "redis"))]
use tower_sessions::session_store::ExpiredDeletion;
#[cfg(feature = "redis")]
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
#[cfg(all(feature = "postgres", not(feature = "redis")))]
use tower_sessions_sqlx_store::PostgresStore;
#[cfg(all(feature = "sqlite", not(feature = "redis")))]
use tower_sessions_sqlx_store::SqliteStore;

mod app;
mod auth;
mod bookmarks;
mod collections;
mod common;
mod entries;
mod feeds;
mod profiles;

const CRON_CLEANUP: &str = "0 0 * * * *";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = colette_config::load_config()?;

    #[cfg(feature = "postgres")]
    let pool = colette_postgres::create_database(&config.database_url).await?;
    #[cfg(feature = "sqlite")]
    let pool = colette_sqlite::create_database(&config.database_url).await?;

    #[cfg(feature = "redis")]
    let (session_store, cleanup) = {
        let Some(redis_url) = config.redis_url.clone() else {
            panic!("\"REDIS_URL\" not set")
        };
        let pool = RedisPool::new(RedisConfig::from_url(&redis_url)?, None, None, None, 1)?;

        let store = RedisStore::new(pool.clone());

        let conn = pool.connect();
        pool.wait_for_connect().await?;

        (store, conn)
    };
    #[cfg(not(feature = "redis"))]
    let (session_store, cleanup) = {
        #[cfg(feature = "postgres")]
        let store = PostgresStore::new(pool.clone());
        #[cfg(feature = "sqlite")]
        let store = SqliteStore::new(pool.clone());

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

    #[cfg(feature = "postgres")]
    let (
        bookmarks_repository,
        collections_repository,
        entries_repository,
        feeds_repository,
        profiles_repository,
        users_repository,
    ) = (
        Arc::new(BookmarksPostgresRepository::new(pool.clone())),
        Arc::new(CollectionsPostgresRepository::new(pool.clone())),
        Arc::new(EntriesPostgresRepository::new(pool.clone())),
        Arc::new(FeedsPostgresRepository::new(pool.clone())),
        Arc::new(ProfilesPostgresRepository::new(pool.clone())),
        Arc::new(UsersPostgresRepository::new(pool)),
    );
    #[cfg(feature = "sqlite")]
    let (
        bookmarks_repository,
        collections_repository,
        entries_repository,
        feeds_repository,
        profiles_repository,
        users_repository,
    ) = (
        Arc::new(BookmarksSqliteRepository::new(pool.clone())),
        Arc::new(CollectionsSqliteRepository::new(pool.clone())),
        Arc::new(EntriesSqliteRepository::new(pool.clone())),
        Arc::new(FeedsSqliteRepository::new(pool.clone())),
        Arc::new(ProfilesSqliteRepository::new(pool.clone())),
        Arc::new(UsersSqliteRepository::new(pool)),
    );

    let downloader = Arc::new(DefaultDownloader {});

    let feed_extractor = Arc::new(DefaultFeedExtractor { options: None });
    let feed_postprocessor = Arc::new(DefaultFeedPostprocessor {});

    let feed_scraper = Arc::new(FeedScraper::new(
        register_feed_plugins(
            downloader.clone(),
            feed_extractor.clone(),
            feed_postprocessor.clone(),
        ),
        downloader.clone(),
        feed_extractor,
        feed_postprocessor,
    ));

    let bookmark_extractor = Arc::new(DefaultBookmarkExtractor::new(None));
    let bookmark_postprocessor = Arc::new(DefaultBookmarkPostprocessor {});

    let bookmark_scraper = Arc::new(BookmarkScraper::new(
        register_bookmark_plugins(
            downloader.clone(),
            bookmark_extractor.clone(),
            bookmark_postprocessor.clone(),
        ),
        downloader,
        bookmark_extractor,
        bookmark_postprocessor,
    ));

    let scheduler = JobScheduler::new().await?;

    if config.refresh_enabled {
        let schedule = Schedule::from_str(&config.cron_refresh)?;

        {
            let feed_scraper = feed_scraper.clone();
            let feeds_repository = feeds_repository.clone();
            let profiles_repository = profiles_repository.clone();

            scheduler
                .add(Job::new_async(schedule.clone(), move |_id, _scheduler| {
                    let refresh_task = RefreshTask::new(
                        feed_scraper.clone(),
                        feeds_repository.clone(),
                        profiles_repository.clone(),
                    );

                    Box::pin(async move {
                        let _ = refresh_task.run().await;
                    })
                })?)
                .await?;
        }
    }

    {
        let feeds_repository = feeds_repository.clone();

        scheduler
            .add(Job::new_async(CRON_CLEANUP, move |_id, _scheduler| {
                let cleanup_task = CleanupTask::new(feeds_repository.clone());

                Box::pin(async move {
                    let _ = cleanup_task.run().await;
                })
            })?)
            .await?;
    }

    scheduler.start().await?;

    let argon_hasher = Arc::new(Argon2Hasher {});

    let auth_service =
        AuthService::new(users_repository, profiles_repository.clone(), argon_hasher);
    let bookmarks_service = BookmarksService::new(bookmarks_repository, bookmark_scraper);
    let collections_service = CollectionsService::new(collections_repository);
    let entries_service = EntriesService::new(entries_repository);
    let feeds_service = FeedsService::new(feeds_repository, feed_scraper);
    let profiles_service = ProfilesService::new(profiles_repository);

    let state = common::Context {
        auth_service: auth_service.into(),
        bookmark_service: bookmarks_service.into(),
        collections_service: collections_service.into(),
        entries_service: entries_service.into(),
        feeds_service: feeds_service.into(),
        profiles_service: profiles_service.into(),
    };

    App::new(state, config, session_store).start().await?;

    cleanup.await??;

    Ok(())
}
