use std::{borrow::Cow, error::Error, ops::RangeFull, str::FromStr, sync::Arc};

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::{CronStream, Schedule};
use apalis_redis::RedisStorage;
use api::{
    auth::{AuthApi, AuthState},
    backup::{BackupApi, BackupState},
    bookmark::{BookmarkApi, BookmarkState},
    common::BaseError,
    feed::{FeedApi, FeedState},
    feed_entry::{FeedEntryApi, FeedEntryState},
    folder::{FolderApi, FolderState},
    library::{LibraryApi, LibraryState},
    tag::{TagApi, TagState},
};
use axum::{
    http::{HeaderValue, Method, header},
    routing,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, feed::FeedService,
    feed_entry::FeedEntryService, folder::FolderService, library::LibraryService, tag::TagService,
};
use colette_http::HyperClient;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use job::{
    archive_thumbnail, import_bookmarks, import_feeds, refresh_feeds, scrape_bookmark, scrape_feed,
};
use object_store::aws::AmazonS3Builder;
use repository::{
    backup::PostgresBackupRepository, bookmark::PostgresBookmarkRepository,
    feed::PostgresFeedRepository, feed_entry::PostgresFeedEntryRepository,
    folder::PostgresFolderRepository, library::PostgresLibraryRepository,
    tag::PostgresTagRepository, user::PostgresUserRepository,
};
use serde::{Deserialize, Deserializer};
use session::RedisStore;
use sqlx::{Pool, Postgres};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use url::Url;
use utoipa::{OpenApi, openapi::Server};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

mod api;
mod job;
mod repository;
mod session;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(BaseError)))]
struct ApiDoc;

#[derive(Debug, Clone, serde::Deserialize)]
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
    pub user_agent: Option<String>,
    pub proxy_url: Option<Url>,
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

    let http_client = {
        use hyper_rustls::HttpsConnectorBuilder;
        use hyper_util::rt::TokioExecutor;

        let https = HttpsConnectorBuilder::new()
            .with_webpki_roots()
            .https_only()
            .enable_http2()
            .build();
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new()).build(https);

        HyperClient::new(client)
    };

    let bucket_url = app_config
        .bucket_endpoint_url
        .origin()
        .ascii_serialization();

    let bucket = AmazonS3Builder::new()
        .with_region(app_config.aws_region)
        .with_endpoint(&bucket_url)
        .with_bucket_name(app_config.bucket_name)
        .with_access_key_id(app_config.aws_access_key_id)
        .with_secret_access_key(app_config.aws_secret_access_key)
        .with_allow_http(true)
        .build()?;

    let redis = redis::Client::open(app_config.redis_url)?;
    let redis_manager = redis.get_connection_manager().await?;

    let scrape_feed_storage = RedisStorage::new(redis_manager.clone());
    let scrape_bookmark_storage = RedisStorage::new(redis_manager.clone());
    let archive_thumbnail_storage = RedisStorage::new(redis_manager.clone());
    let import_feeds_storage = RedisStorage::new(redis_manager.clone());
    let import_bookmarks_storage = RedisStorage::new(redis_manager);

    let auth_service = Arc::new(AuthService::new(PostgresUserRepository::new(pool.clone())));
    let backup_service = Arc::new(BackupService::new(
        PostgresBackupRepository::new(pool.clone()),
        feed_repository.clone(),
        bookmark_repository.clone(),
        folder_repository.clone(),
        Arc::new(Mutex::new(import_feeds_storage.clone())),
        Arc::new(Mutex::new(import_bookmarks_storage.clone())),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository,
        http_client.clone(),
        bucket,
        Arc::new(Mutex::new(archive_thumbnail_storage.clone())),
        register_bookmark_plugins(http_client.clone()),
        bucket_url,
    ));
    // let collection_service = Arc::new(CollectionService::new(PostgresCollectionRepository::new(
    //     pool.clone(),
    // )));
    let feed_service = Arc::new(FeedService::new(
        feed_repository,
        http_client.clone(),
        register_feed_plugins(http_client),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(PostgresFeedEntryRepository::new(
        pool.clone(),
    )));
    let folder_service = Arc::new(FolderService::new(folder_repository));
    let library_service = Arc::new(LibraryService::new(PostgresLibraryRepository::new(
        pool.clone(),
    )));
    // let smart_feed_service = Arc::new(SmartFeedService::new(PostgresSmartFeedRepository::new(
    //     pool.clone(),
    // )));
    let tag_service = Arc::new(TagService::new(PostgresTagRepository::new(pool)));

    let scrape_feed_worker = WorkerBuilder::new("scrape-feed")
        .enable_tracing()
        .concurrency(5)
        .data(feed_service.clone())
        .backend(scrape_feed_storage.clone())
        .build_fn(scrape_feed::run);

    let scrape_bookmark_worker = WorkerBuilder::new("scrape-bookmark")
        .enable_tracing()
        .concurrency(5)
        .data(bookmark_service.clone())
        .backend(scrape_bookmark_storage.clone())
        .build_fn(scrape_bookmark::run);

    let schedule = Schedule::from_str(&app_config.cron_refresh)?;

    let refresh_feeds_worker = WorkerBuilder::new("refresh-feeds")
        .enable_tracing()
        .data(refresh_feeds::State::new(
            feed_service.clone(),
            Arc::new(Mutex::new(scrape_feed_storage.clone())),
        ))
        .backend(CronStream::new(schedule))
        .build_fn(refresh_feeds::run);

    let archive_thumbnail_worker = WorkerBuilder::new("archive_thumbnail")
        .enable_tracing()
        .concurrency(5)
        .data(bookmark_service.clone())
        .backend(archive_thumbnail_storage)
        .build_fn(archive_thumbnail::run);

    let import_feeds_worker = WorkerBuilder::new("import-feeds")
        .enable_tracing()
        .data(scrape_feed_storage)
        .backend(import_feeds_storage)
        .build_fn(import_feeds::run);

    let import_bookmarks_worker = WorkerBuilder::new("import-bookmarks")
        .enable_tracing()
        .data(scrape_bookmark_storage)
        .backend(import_bookmarks_storage)
        .build_fn(import_bookmarks::run);

    let monitor = Monitor::new()
        .register(scrape_feed_worker)
        .register(scrape_bookmark_worker)
        .register(refresh_feeds_worker)
        .register(archive_thumbnail_worker)
        .register(import_feeds_worker)
        .register(import_bookmarks_worker)
        .run();

    let redis_conn = redis.get_multiplexed_async_connection().await?;
    let session_store = RedisStore::new(redis_conn);

    let (api, mut openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest(
            &app_config.api_prefix,
            OpenApiRouter::new()
                .nest("/auth", AuthApi::router())
                .with_state(AuthState::new(auth_service))
                .nest("/backups", BackupApi::router())
                .with_state(BackupState::new(backup_service))
                .nest("/bookmarks", BookmarkApi::router())
                .with_state(BookmarkState::new(bookmark_service))
                // .nest("/collections", CollectionApi::router())
                // .with_state(CollectionState::new(collection_service))
                .nest("/feedEntries", FeedEntryApi::router())
                .with_state(FeedEntryState::new(feed_entry_service))
                .nest("/feeds", FeedApi::router())
                .with_state(FeedState::new(feed_service))
                .nest("/folders", FolderApi::router())
                .with_state(FolderState::new(folder_service))
                .nest("/library", LibraryApi::router())
                .with_state(LibraryState::new(library_service))
                // .nest("/smartFeeds", SmartFeedApi::router())
                // .with_state(SmartFeedState::new(smart_feed_service))
                .nest("/tags", TagApi::router())
                .with_state(TagState::new(tag_service)),
        )
        .split_for_parts();

    openapi.info.title = "Colette API".to_owned();
    openapi.servers = Some(vec![Server::new(&app_config.api_prefix)]);

    openapi.paths.paths = openapi
        .paths
        .paths
        .drain(RangeFull)
        .map(|(k, v)| (k.replace(&app_config.api_prefix, ""), v))
        .collect();

    let mut api = api
        .merge(Scalar::with_url(
            format!("{}/doc", &app_config.api_prefix),
            openapi.clone(),
        ))
        .route(
            &format!("{}/openapi.json", &app_config.api_prefix),
            routing::get(|| async move { openapi.to_pretty_json().unwrap() }),
        )
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
