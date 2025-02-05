use std::{borrow::Cow, error::Error, str::FromStr, sync::Arc, time::Duration};

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronStream, Schedule};
use apalis_redis::RedisStorage;
use axum::http::{header, HeaderValue, Method};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
    feed_entry::FeedEntryState, folder::FolderState, library::LibraryState, tag::TagState, Api,
    ApiState,
};
use colette_archiver::ThumbnailArchiver;
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, feed::FeedService,
    feed_entry::FeedEntryService, folder::FolderService, library::LibraryService,
    scraper::ScraperService, tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_repository::{
    PostgresBackupRepository, PostgresBookmarkRepository, PostgresFeedEntryRepository,
    PostgresFeedRepository, PostgresFolderRepository, PostgresLibraryRepository,
    PostgresScraperRepository, PostgresTagRepository, PostgresUserRepository,
};
use colette_scraper::{
    bookmark::DefaultBookmarkScraper,
    feed::{DefaultFeedDetector, DefaultFeedScraper},
};
use colette_task::{
    archive_thumbnail, import_bookmarks, import_feeds, refresh_feeds, scrape_bookmark, scrape_feed,
};
use redis::Client;
use reqwest::{ClientBuilder, Url};
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use serde::{Deserialize, Deserializer};
use session::RedisStore;
use sqlx::{Pool, Postgres};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{cookie::time, Expiry, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod session;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub aws_access_key_id: Cow<'static, str>,
    pub aws_secret_access_key: Cow<'static, str>,
    #[serde(default = "default_aws_region")]
    pub aws_region: Cow<'static, str>,
    #[serde(default = "default_bucket_name")]
    pub bucket_name: Cow<'static, str>,
    #[serde(default = "default_bucket_endpoint_url")]
    pub bucket_endpoint_url: Url,
    #[serde(deserialize_with = "string_to_vec", default = "default_origin_urls")]
    pub origin_urls: Vec<String>,
    #[serde(default = "default_refresh_enabled")]
    pub refresh_enabled: bool,
    #[serde(default = "default_cron_refresh")]
    pub cron_refresh: String,
    #[serde(default = "default_api_prefix")]
    pub api_prefix: String,
}

fn default_host() -> String {
    "0.0.0.0".to_owned()
}

fn default_port() -> u16 {
    8000
}

fn default_aws_region() -> Cow<'static, str> {
    Cow::Borrowed("us-east-1")
}

fn default_bucket_name() -> Cow<'static, str> {
    Cow::Borrowed("colette")
}

fn default_bucket_endpoint_url() -> Url {
    "http://localhost:9000".parse().unwrap()
}

pub fn string_to_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    let parsed = value.split(',').map(|s| s.trim().to_owned()).collect();

    Ok(parsed)
}

fn default_origin_urls() -> Vec<String> {
    Vec::new()
}

fn default_refresh_enabled() -> bool {
    true
}

fn default_cron_refresh() -> String {
    "0 */15 * * * *".to_owned()
}

fn default_api_prefix() -> String {
    "/api/v1".to_owned()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::from_default_env())
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    #[cfg(not(debug_assertions))]
    {
        use tracing_subscriber::Layer;
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
            )
            .init();
    }

    let app_config = envy::from_env::<AppConfig>()?;

    let pool = Pool::<Postgres>::connect(&app_config.database_url).await?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    let bookmark_repository = PostgresBookmarkRepository::new(pool.clone());
    let feed_repository = PostgresFeedRepository::new(pool.clone());
    let folder_repository = PostgresFolderRepository::new(pool.clone());

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .build()?;
    let feed_plugin_registry = Arc::new(register_feed_plugins(
        client.clone(),
        DefaultFeedScraper::new(client.clone()),
    ));
    let bookmark_plugin_registry = Arc::new(register_bookmark_plugins(
        client.clone(),
        DefaultBookmarkScraper::new(client.clone()),
    ));

    let bucket = {
        let region = Region::Custom {
            region: app_config.aws_region.into_owned(),
            endpoint: app_config
                .bucket_endpoint_url
                .origin()
                .ascii_serialization(),
        };
        let credentials = Credentials::new(
            Some(&app_config.aws_access_key_id),
            Some(&app_config.aws_secret_access_key),
            None,
            None,
            None,
        )?;

        let bucket = Bucket::new(&app_config.bucket_name, region.clone(), credentials.clone())?
            .with_path_style();

        let exists = bucket.exists().await?;
        if !exists {
            Bucket::create_with_path_style(
                &app_config.bucket_name,
                region,
                credentials,
                BucketConfiguration::private(),
            )
            .await?;
        }

        bucket
    };

    let auth_service = Arc::new(AuthService::new(PostgresUserRepository::new(pool.clone())));
    let backup_service = Arc::new(BackupService::new(
        PostgresBackupRepository::new(pool.clone()),
        feed_repository.clone(),
        bookmark_repository.clone(),
        folder_repository.clone(),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository,
        bookmark_plugin_registry.clone(),
        ThumbnailArchiver::new(client.clone(), bucket),
    ));
    // let collection_service = Arc::new(CollectionService::new(PostgresCollectionRepository::new(
    //     pool.clone(),
    // )));
    let feed_service = Arc::new(FeedService::new(
        feed_repository,
        Box::new(DefaultFeedDetector::new(client)),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(PostgresFeedEntryRepository::new(
        pool.clone(),
    )));
    let folder_service = Arc::new(FolderService::new(folder_repository));
    let library_service = Arc::new(LibraryService::new(PostgresLibraryRepository::new(
        pool.clone(),
    )));
    let scraper_service = Arc::new(ScraperService::new(
        PostgresScraperRepository::new(pool.clone()),
        feed_plugin_registry,
        bookmark_plugin_registry,
    ));
    // let smart_feed_service = Arc::new(SmartFeedService::new(PostgresSmartFeedRepository::new(
    //     pool.clone(),
    // )));
    let tag_service = Arc::new(TagService::new(PostgresTagRepository::new(pool)));

    let redis = Client::open(app_config.redis_url)?;
    let redis_manager = redis.get_connection_manager().await?;

    let scrape_feed_storage = RedisStorage::new(redis_manager.clone());
    let scrape_feed_worker = WorkerBuilder::new("scrape-feed")
        .enable_tracing()
        .concurrency(5)
        .data(scraper_service.clone())
        .backend(scrape_feed_storage.clone())
        .build_fn(scrape_feed::run);

    let scrape_bookmark_storage = RedisStorage::new(redis_manager.clone());
    let scrape_bookmark_worker = WorkerBuilder::new("scrape-bookmark")
        .enable_tracing()
        .concurrency(5)
        .data(scraper_service)
        .backend(scrape_bookmark_storage.clone())
        .build_fn(scrape_bookmark::run);

    let import_feeds_storage = RedisStorage::new(redis_manager.clone());
    let import_feeds_worker = WorkerBuilder::new("import-feeds")
        .enable_tracing()
        .data(scrape_feed_storage.clone())
        .backend(import_feeds_storage.clone())
        .build_fn(import_feeds::run);

    let import_bookmarks_storage = RedisStorage::new(redis_manager.clone());
    let import_bookmarks_worker = WorkerBuilder::new("import-bookmarks")
        .enable_tracing()
        .data(scrape_bookmark_storage)
        .backend(import_bookmarks_storage.clone())
        .build_fn(import_bookmarks::run);

    let schedule = Schedule::from_str(&app_config.cron_refresh)?;

    let refresh_feeds_worker = WorkerBuilder::new("refresh-feeds")
        .enable_tracing()
        .data(refresh_feeds::State::new(
            feed_service.clone(),
            Arc::new(Mutex::new(scrape_feed_storage)),
        ))
        .backend(CronStream::new(schedule))
        .build_fn(refresh_feeds::run);

    let archive_thumbnail_storage = RedisStorage::new(redis_manager);
    let archive_thumbnail_worker = WorkerBuilder::new("archive_thumbnail")
        .enable_tracing()
        .concurrency(5)
        .data(bookmark_service.clone())
        .backend(archive_thumbnail_storage.clone())
        .build_fn(archive_thumbnail::run);

    let monitor = Monitor::new()
        .register(scrape_feed_worker)
        .register(scrape_bookmark_worker)
        .register(import_feeds_worker)
        .register(import_bookmarks_worker)
        .register(refresh_feeds_worker)
        .register(archive_thumbnail_worker)
        .run();

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(
            backup_service,
            Arc::new(Mutex::new(import_feeds_storage)),
            Arc::new(Mutex::new(import_bookmarks_storage)),
        ),
        BookmarkState::new(
            bookmark_service,
            Arc::new(Mutex::new(archive_thumbnail_storage)),
        ),
        // CollectionState::new(collection_service),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        FolderState::new(folder_service),
        LibraryState::new(library_service),
        // SmartFeedState::new(smart_feed_service),
        TagState::new(tag_service),
    );

    let redis_conn = redis.get_multiplexed_async_connection().await?;
    let session_store = RedisStore::new(redis_conn);

    let mut api = Api::new(&api_state, &app_config.api_prefix)
        .build()
        .with_state(api_state)
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(time::Duration::days(1))),
        )
        .layer(TraceLayer::new_for_http())
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ));

    if !app_config.origin_urls.is_empty() {
        let origins = app_config
            .origin_urls
            .iter()
            .filter_map(|e| e.parse::<HeaderValue>().ok())
            .collect::<Vec<_>>();

        api = api.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(origins)
                .allow_headers([header::CONTENT_TYPE])
                .allow_credentials(true),
        )
    }

    let listener = TcpListener::bind(format!("{}:{}", app_config.host, app_config.port)).await?;
    let server = axum::serve(listener, api);

    let _ = tokio::join!(monitor, server);

    Ok(())
}
