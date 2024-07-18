#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("features \"postgres\" and \"sqlite\" are mutually exclusive");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Either feature \"postgres\" or \"sqlite\" must be enabled");

use std::{collections::HashMap, error::Error, str::FromStr, sync::Arc};

use auth::Api as Auth;
use axum::{
    http::{header, HeaderValue, Method},
    routing, Router,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use bookmarks::Api as Bookmarks;
use chrono::Utc;
use colette_core::{
    auth::AuthService,
    bookmarks::BookmarksService,
    collections::CollectionsService,
    entries::EntriesService,
    feeds::{FeedCreateData, FeedsRepository, FeedsService, ProcessedFeed},
    profiles::{ProfilesRepository, ProfilesService},
    utils::{
        scraper::Scraper,
        task::{self, Task},
    },
};
use colette_password::Argon2Hasher;
#[cfg(feature = "postgres")]
use colette_postgres::{
    BookmarksPostgresRepository, CollectionsPostgresRepository, EntriesPostgresRepository,
    FeedsPostgresRepository, ProfilesPostgresRepository, UsersPostgresRepository,
};
use colette_scraper::{
    atom_extractor_options, DefaultDownloader, DefaultFeedExtractor, DefaultFeedPostprocessor,
    FeedScraper, PluginRegistry,
};
#[cfg(feature = "sqlite")]
use colette_sqlite::{
    BookmarksSqliteRepository, CollectionsSqliteRepository, EntriesSqliteRepository,
    FeedsSqliteRepository, ProfilesSqliteRepository, UsersSqliteRepository,
};
use collections::Api as Collections;
use common::{BookmarkList, CollectionList, EntryList, FeedList, ProfileList};
use cron::Schedule;
use entries::Api as Entries;
use feeds::Api as Feeds;
use futures::stream::StreamExt;
use profiles::Api as Profiles;
use tokio::{net::TcpListener, sync::Semaphore};
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::CorsLayer;
#[cfg(not(feature = "redis"))]
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
#[cfg(feature = "redis")]
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};
#[cfg(all(feature = "postgres", not(feature = "redis")))]
use tower_sessions_sqlx_store::PostgresStore;
#[cfg(all(feature = "sqlite", not(feature = "redis")))]
use tower_sessions_sqlx_store::SqliteStore;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

mod auth;
mod bookmarks;
mod collections;
mod common;
mod entries;
mod feeds;
mod profiles;

const CRON_CLEANUP: &str = "0 0 * * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../../packages/web/dist"]
struct Asset;

#[derive(utoipa::OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:8000/api/v1")
    ),
    nest(
        (path = "/auth", api = Auth),
        (path = "/bookmarks", api = Bookmarks),
        (path = "/collections", api = Collections),
        (path = "/entries", api = Entries),
        (path = "/feeds", api = Feeds),
        (path = "/profiles", api = Profiles)
    ),
    components(schemas(common::BaseError, BookmarkList, CollectionList, EntryList, FeedList, ProfileList))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = colette_config::load_config()?;

    #[cfg(feature = "postgres")]
    let pool = colette_postgres::create_database(&config.database_url).await?;
    #[cfg(feature = "sqlite")]
    let pool = colette_sqlite::create_database(&config.database_url).await?;

    #[cfg(feature = "redis")]
    let (session_store, cleanup) = {
        let Some(redis_url) = config.redis_url else {
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

    let downloader = DefaultDownloader {};
    let feed_extractor = DefaultFeedExtractor {
        options: atom_extractor_options(),
    };
    let feed_postprocessor = DefaultFeedPostprocessor {};

    let feed_registry = PluginRegistry {
        downloaders: HashMap::new(),
        extractors: HashMap::new(),
        postprocessors: HashMap::new(),
    };
    let feed_scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync> = Arc::new(FeedScraper::new(
        feed_registry,
        Arc::new(downloader),
        Arc::new(feed_extractor),
        Arc::new(feed_postprocessor),
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
    let bookmarks_service = BookmarksService::new(bookmarks_repository);
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

    let mut app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .merge(Scalar::with_url("/doc", ApiDoc::openapi()))
                .route(
                    "/openapi.json",
                    routing::get(|| async { ApiDoc::openapi().to_pretty_json().unwrap() }),
                )
                .merge(Auth::router())
                .merge(Bookmarks::router())
                .merge(Collections::router())
                .merge(Entries::router())
                .merge(Feeds::router())
                .merge(Profiles::router())
                .with_state(state),
        )
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        );

    if !config.origin_urls.is_empty() {
        let origins: Vec<HeaderValue> = config
            .origin_urls
            .iter()
            .filter_map(|e| e.parse::<HeaderValue>().ok())
            .collect();

        app = app.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(origins)
                .allow_headers([header::CONTENT_TYPE])
                .allow_credentials(true),
        )
    }

    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    axum::serve(listener, app).await?;

    cleanup.await??;

    Ok(())
}

pub struct RefreshTask {
    scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync>,
    feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
    profiles_repo: Arc<dyn ProfilesRepository + Send + Sync>,
}

impl RefreshTask {
    pub fn new(
        scraper: Arc<dyn Scraper<ProcessedFeed> + Send + Sync>,
        feeds_repo: Arc<dyn FeedsRepository + Send + Sync>,
        profiles_repo: Arc<dyn ProfilesRepository + Send + Sync>,
    ) -> Self {
        Self {
            scraper,
            feeds_repo,
            profiles_repo,
        }
    }

    async fn refresh(&self, feed_id: i64, url: String) {
        println!("{}: refreshing {}", Utc::now().to_rfc3339(), url);

        let feed = self.scraper.scrape(&url).await.unwrap();

        let mut profiles_stream = self.profiles_repo.iterate(feed_id);

        while let Some(Ok(profile_id)) = profiles_stream.next().await {
            let data = FeedCreateData {
                url: url.clone(),
                feed: feed.clone(),
                profile_id,
            };
            self.feeds_repo.create(data).await.unwrap();
        }
    }
}

#[async_trait::async_trait]
impl Task for RefreshTask {
    async fn run(&self) -> Result<(), task::Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self.feeds_repo.iterate();

        let tasks = feeds_stream
            .map(|item| {
                let semaphore = semaphore.clone();

                async move {
                    let _ = semaphore.acquire().await.unwrap();

                    if let Ok((feed_id, url)) = item {
                        self.refresh(feed_id, url).await
                    }
                }
            })
            .buffer_unordered(5);

        tasks.for_each(|_| async {}).await;

        Ok(())
    }
}

pub struct CleanupTask {
    repo: Arc<dyn FeedsRepository + Send + Sync>,
}

impl CleanupTask {
    pub fn new(repo: Arc<dyn FeedsRepository + Send + Sync>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl Task for CleanupTask {
    async fn run(&self) -> Result<(), task::Error> {
        self.repo
            .cleanup()
            .await
            .map_err(|e| task::Error(e.into()))?;

        Ok(())
    }
}
