#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("features \"postgres\" and \"sqlite\" are mutually exclusive");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("Either feature \"postgres\" or \"sqlite\" must be enabled");

use std::{collections::HashMap, env, error::Error, str::FromStr, sync::Arc};

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
    utils::{scraper::Scraper, task::Task},
};
use colette_password::Argon2Hasher;
#[cfg(feature = "postgres")]
use colette_postgres::{
    BookmarksPostgresRepository, CollectionsPostgresRepository, EntriesPostgresRepository,
    FeedsPostgresRepository, ProfilesPostgresRepository, UsersPostgresRepository,
};
use colette_scraper::{
    AtomExtractorOptions, DefaultDownloader, DefaultFeedExtractor, DefaultFeedPostprocessor,
    ExtractorOptions, FeedScraper, PluginRegistry,
};
#[cfg(feature = "sqlite")]
use colette_sqlite::{
    iterate_feeds, iterate_profiles, BookmarksSqliteRepository, CollectionsSqliteRepository,
    EntriesSqliteRepository, FeedsSqliteRepository, ProfilesSqliteRepository,
    UsersSqliteRepository,
};
use collections::Api as Collections;
use common::{BookmarkList, CollectionList, EntryList, FeedList, ProfileList};
use cron::Schedule;
use entries::Api as Entries;
use feeds::Api as Feeds;
use futures::stream::StreamExt;
use profiles::Api as Profiles;
#[cfg(not(feature = "redis"))]
use tokio::task;
use tokio::{net::TcpListener, sync::Semaphore};
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::CorsLayer;
#[cfg(not(feature = "redis"))]
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
#[cfg(feature = "redis")]
use tower_sessions_redis_store::fred::prelude::*;
#[cfg(feature = "redis")]
use tower_sessions_redis_store::RedisStore;
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

const DEFAULT_PORT: u32 = 8000;
const DEFAULT_CRON_REFRESH: &str = "0 */15 * * * * *";

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
    components(schemas(common::BaseError, common::ValidationError, BookmarkList, CollectionList, EntryList, FeedList, ProfileList))
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port = env::var("PORT")
        .map(|e| e.parse::<u32>())
        .unwrap_or(Ok(DEFAULT_PORT))?;
    let database_url = env::var("DATABASE_URL")?;
    let origin_urls = env::var("ORIGIN_URLS").ok();
    let cron_refresh = env::var("CRON_REFRESH").unwrap_or(String::from(DEFAULT_CRON_REFRESH));

    #[cfg(feature = "redis")]
    let (redis_pool, redis_conn) = {
        let redis_url = env::var("REDIS_URL")?;
        let pool = RedisPool::new(RedisConfig::from_url(&redis_url)?, None, None, None, 1)?;

        let conn = pool.connect();
        pool.wait_for_connect().await?;

        (pool, conn)
    };
    #[cfg(feature = "postgres")]
    let pool = colette_postgres::create_database(&database_url).await?;
    #[cfg(feature = "sqlite")]
    let pool = colette_sqlite::create_database(&database_url).await?;

    #[cfg(feature = "redis")]
    let session_store = RedisStore::new(redis_pool);
    #[cfg(all(feature = "postgres", not(feature = "redis")))]
    let session_store = PostgresStore::new(pool.clone());
    #[cfg(all(feature = "sqlite", not(feature = "redis")))]
    let session_store = SqliteStore::new(pool.clone());

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
        options: ExtractorOptions {
            ..AtomExtractorOptions::default().inner()
        },
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

    let schedule = Schedule::from_str(&cron_refresh)?;
    let scheduler = JobScheduler::new().await?;

    {
        let feed_scraper = feed_scraper.clone();
        let feeds_repository = feeds_repository.clone();
        let profiles_repository = profiles_repository.clone();

        scheduler
            .add(Job::new_async(schedule, move |_id, _scheduler| {
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

    scheduler.start().await?;

    let argon_hasher = Arc::new(Argon2Hasher::default());

    let auth_service =
        AuthService::new(users_repository, profiles_repository.clone(), argon_hasher);
    let bookmarks_service = BookmarksService::new(bookmarks_repository);
    let collections_service = CollectionsService::new(collections_repository);
    let entries_service = EntriesService::new(entries_repository);
    let feeds_service = FeedsService::new(feeds_repository, feed_scraper);
    let profiles_service = ProfilesService::new(profiles_repository);

    #[cfg(not(feature = "redis"))]
    let deletion_task = {
        session_store.migrate().await?;

        task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        )
    };

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

    if let Some(origin_urls) = origin_urls {
        let mut origins: Vec<HeaderValue> = vec![];
        for part in origin_urls.split(",") {
            let origin = part.parse::<HeaderValue>()?;
            origins.push(origin);
        }

        app = app.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(origins)
                .allow_headers([header::CONTENT_TYPE])
                .allow_credentials(true),
        )
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    #[cfg(feature = "redis")]
    redis_conn.await??;

    #[cfg(not(feature = "redis"))]
    deletion_task.await??;

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
    async fn run(&self) {
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
    }
}
