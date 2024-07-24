#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("features \"postgres\" and \"sqlite\" are mutually exclusive");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Either feature \"postgres\" or \"sqlite\" must be enabled");

use std::{error::Error, sync::Arc};

use app::App;
use colette_backup::OpmlManager;
use colette_core::{
    auth::AuthService,
    bookmarks::{BookmarksRepository, BookmarksService},
    collections::{CollectionsRepository, CollectionsService},
    entries::{EntriesRepository, EntriesService},
    feeds::{FeedsRepository, FeedsService},
    profiles::{ProfilesRepository, ProfilesService},
    tags::{TagsRepository, TagsService},
    users::UsersRepository,
    utils::task::Task,
};
use colette_password::Argon2Hasher;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_scraper::{BookmarkScraper, FeedScraper};
use colette_tasks::{CleanupTask, RefreshTask};
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
mod tags;

const CRON_CLEANUP: &str = "0 0 * * * *";

struct Repositories {
    bookmarks: Arc<dyn BookmarksRepository>,
    collections: Arc<dyn CollectionsRepository>,
    entries: Arc<dyn EntriesRepository>,
    feeds: Arc<dyn FeedsRepository>,
    profiles: Arc<dyn ProfilesRepository>,
    tags: Arc<dyn TagsRepository>,
    users: Arc<dyn UsersRepository>,
}

impl Repositories {
    #[cfg(feature = "postgres")]
    fn new_pg(pool: colette_postgres::PgPool) -> Self {
        use colette_postgres::*;
        Self {
            bookmarks: Arc::new(BookmarksPostgresRepository::new(pool.clone())),
            collections: Arc::new(CollectionsPostgresRepository::new(pool.clone())),
            entries: Arc::new(EntriesPostgresRepository::new(pool.clone())),
            feeds: Arc::new(FeedsPostgresRepository::new(pool.clone())),
            profiles: Arc::new(ProfilesPostgresRepository::new(pool.clone())),
            tags: Arc::new(TagsPostgresRepository::new(pool.clone())),
            users: Arc::new(UsersPostgresRepository::new(pool)),
        }
    }

    #[cfg(feature = "sqlite")]
    fn new_sqlite(pool: colette_sqlite::SqlitePool) -> Self {
        use colette_sqlite::*;
        Self {
            bookmarks: Arc::new(BookmarksSqliteRepository::new(pool.clone())),
            collections: Arc::new(CollectionsSqliteRepository::new(pool.clone())),
            entries: Arc::new(EntriesSqliteRepository::new(pool.clone())),
            feeds: Arc::new(FeedsSqliteRepository::new(pool.clone())),
            profiles: Arc::new(ProfilesSqliteRepository::new(pool.clone())),
            tags: Arc::new(TagsSqliteRepository::new(pool.clone())),
            users: Arc::new(UsersSqliteRepository::new(pool)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = colette_config::load_config()?;

    #[cfg(feature = "postgres")]
    let (pool, repositories) = {
        let pool = colette_postgres::create_database(&config.database_url).await?;

        (pool.clone(), Repositories::new_pg(pool))
    };
    #[cfg(feature = "sqlite")]
    let (pool, repositories) = {
        let pool = colette_sqlite::create_database(&config.database_url).await?;

        (pool.clone(), Repositories::new_sqlite(pool))
    };

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

    let feed_scraper = Arc::new(FeedScraper::new(register_feed_plugins()));

    let scheduler = JobScheduler::new().await?;

    if config.refresh_enabled {
        let feed_scraper = feed_scraper.clone();
        let feeds_repository = repositories.feeds.clone();
        let profiles_repository = repositories.profiles.clone();

        scheduler
            .add(Job::new_async(
                config.cron_refresh.as_ref(),
                move |_id, _scheduler| {
                    let refresh_task = RefreshTask::new(
                        feed_scraper.clone(),
                        feeds_repository.clone(),
                        profiles_repository.clone(),
                    );

                    Box::pin(async move {
                        let _ = refresh_task.run().await;
                    })
                },
            )?)
            .await?;
    }

    {
        let feeds_repository = repositories.feeds.clone();

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

    let state = common::Context {
        auth_service: AuthService::new(
            repositories.users,
            repositories.profiles.clone(),
            Arc::new(Argon2Hasher {}),
        )
        .into(),
        bookmark_service: BookmarksService::new(
            repositories.bookmarks,
            Arc::new(BookmarkScraper::new(register_bookmark_plugins())),
        )
        .into(),
        collections_service: CollectionsService::new(repositories.collections).into(),
        entries_service: EntriesService::new(repositories.entries).into(),
        feeds_service: FeedsService::new(repositories.feeds, feed_scraper, Arc::new(OpmlManager))
            .into(),
        profiles_service: ProfilesService::new(repositories.profiles).into(),
        tags_service: TagsService::new(repositories.tags).into(),
    };

    App::new(state, config, session_store).start().await?;

    cleanup.await??;

    Ok(())
}
