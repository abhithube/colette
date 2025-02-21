use std::{borrow::Cow, error::Error, ops::RangeFull, sync::Arc};

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::CronStream;
use apalis_redis::RedisStorage;
use api::{
    ApiState, api_key::ApiKeyApi, auth::AuthApi, backup::BackupApi, bookmark::BookmarkApi,
    collection::CollectionApi, common::BaseError, feed::FeedApi, feed_entry::FeedEntryApi,
    folder::FolderApi, library::LibraryApi, tag::TagApi,
};
use axum::{
    http::{HeaderValue, Method, header},
    routing,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_core::{
    api_key::ApiKeyService, auth::AuthService, backup::BackupService, bookmark::BookmarkService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    folder::FolderService, library::LibraryService, tag::TagService,
};
use colette_http::ReqwestClient;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use job::{
    archive_thumbnail, import_bookmarks, import_feeds, refresh_feeds, scrape_bookmark, scrape_feed,
};
use object_store::aws::AmazonS3Builder;
use repository::{
    accounts::PostgresAccountRepository, api_key::PostgresApiKeyRepository,
    backup::PostgresBackupRepository, bookmark::PostgresBookmarkRepository,
    collection::PostgresCollectionRepository, feed::PostgresFeedRepository,
    feed_entry::PostgresFeedEntryRepository, folder::PostgresFolderRepository,
    library::PostgresLibraryRepository, tag::PostgresTagRepository, user::PostgresUserRepository,
};
use session::RedisStore;
use sqlx::{Pool, Postgres, postgres::PgConnectOptions};
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
    pub host: Cow<'static, str>,
    #[serde(default = "default_port")]
    pub port: u32,
    pub database_url: Option<Cow<'static, str>>,
    pub redis_url: String,
    #[serde(default = "default_cron_enabled")]
    pub cron_enabled: bool,
    #[serde(default = "default_cron_schedule")]
    pub cron_schedule: Cow<'static, str>,
    pub aws_access_key_id: Cow<'static, str>,
    pub aws_secret_access_key: Cow<'static, str>,
    #[serde(default = "default_aws_region")]
    pub aws_region: Cow<'static, str>,
    #[serde(default = "default_s3_bucket_name")]
    pub s3_bucket_name: Cow<'static, str>,
    #[serde(default = "default_s3_bucket_endpoint_url")]
    pub s3_bucket_endpoint_url: Url,
    #[serde(default = "default_cors_enabled")]
    pub cors_enabled: bool,
    #[serde(default = "default_origin_urls")]
    pub origin_urls: Vec<Cow<'static, str>>,
}

fn default_host() -> Cow<'static, str> {
    "0.0.0.0".into()
}

fn default_port() -> u32 {
    8000
}

fn default_cron_enabled() -> bool {
    true
}

fn default_cron_schedule() -> Cow<'static, str> {
    "0 */15 * * * *".into()
}

fn default_aws_region() -> Cow<'static, str> {
    "us-east-1".into()
}

fn default_s3_bucket_name() -> Cow<'static, str> {
    "colette".into()
}

fn default_s3_bucket_endpoint_url() -> Url {
    "http://localhost:9000".parse().unwrap()
}

fn default_cors_enabled() -> bool {
    false
}

fn default_origin_urls() -> Vec<Cow<'static, str>> {
    Vec::new()
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

    let pool = match app_config.database_url {
        Some(database_url) => Pool::<Postgres>::connect(&database_url).await,
        _ => Pool::<Postgres>::connect_with(PgConnectOptions::new()).await,
    }?;
    sqlx::migrate!("../../migrations").run(&pool).await?;

    let reqwest_client = reqwest::Client::builder().https_only(true).build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let bucket_url = app_config
        .s3_bucket_endpoint_url
        .join(&format!("{}/", app_config.s3_bucket_name))
        .unwrap();

    let bucket = AmazonS3Builder::new()
        .with_region(app_config.aws_region)
        .with_endpoint(
            app_config
                .s3_bucket_endpoint_url
                .origin()
                .ascii_serialization(),
        )
        .with_bucket_name(app_config.s3_bucket_name)
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

    let api_key_service = Arc::new(ApiKeyService::new(PostgresApiKeyRepository::new(
        pool.clone(),
    )));
    let auth_service = Arc::new(AuthService::new(
        PostgresUserRepository::new(pool.clone()),
        PostgresAccountRepository::new(pool.clone()),
    ));
    let backup_service = Arc::new(BackupService::new(
        PostgresBackupRepository::new(pool.clone()),
        Arc::new(Mutex::new(import_feeds_storage.clone())),
        Arc::new(Mutex::new(import_bookmarks_storage.clone())),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        PostgresBookmarkRepository::new(pool.clone()),
        http_client.clone(),
        bucket,
        Arc::new(Mutex::new(archive_thumbnail_storage.clone())),
        register_bookmark_plugins(reqwest_client.clone()),
    ));
    let collection_service = Arc::new(CollectionService::new(PostgresCollectionRepository::new(
        pool.clone(),
    )));
    let feed_service = Arc::new(FeedService::new(
        PostgresFeedRepository::new(pool.clone()),
        http_client.clone(),
        register_feed_plugins(reqwest_client),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(PostgresFeedEntryRepository::new(
        pool.clone(),
    )));
    let folder_service = Arc::new(FolderService::new(PostgresFolderRepository::new(
        pool.clone(),
    )));
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

    let schedule = app_config.cron_schedule.parse()?;

    let archive_thumbnail_worker = WorkerBuilder::new("archive_thumbnail")
        .enable_tracing()
        .concurrency(5)
        .data(bookmark_service.clone())
        .backend(archive_thumbnail_storage)
        .build_fn(archive_thumbnail::run);

    let import_feeds_worker = WorkerBuilder::new("import-feeds")
        .enable_tracing()
        .data(scrape_feed_storage.clone())
        .backend(import_feeds_storage)
        .build_fn(import_feeds::run);

    let import_bookmarks_worker = WorkerBuilder::new("import-bookmarks")
        .enable_tracing()
        .data(scrape_bookmark_storage)
        .backend(import_bookmarks_storage)
        .build_fn(import_bookmarks::run);

    let mut monitor = Monitor::new()
        .register(scrape_feed_worker)
        .register(scrape_bookmark_worker)
        .register(archive_thumbnail_worker)
        .register(import_feeds_worker)
        .register(import_bookmarks_worker);

    if app_config.cron_enabled {
        let refresh_feeds_worker = WorkerBuilder::new("refresh-feeds")
            .enable_tracing()
            .data(refresh_feeds::State::new(
                feed_service.clone(),
                Arc::new(Mutex::new(scrape_feed_storage)),
            ))
            .backend(CronStream::new(schedule))
            .build_fn(refresh_feeds::run);

        monitor = monitor.register(refresh_feeds_worker)
    }

    let monitor = monitor.run();

    let redis_conn = redis.get_multiplexed_async_connection().await?;
    let session_store = RedisStore::new(redis_conn);

    let api_state = ApiState {
        api_key_service,
        auth_service,
        backup_service,
        bookmark_service,
        collection_service,
        feed_service,
        feed_entry_service,
        folder_service,
        library_service,
        tag_service,
        bucket_url,
    };

    let api_prefix = "/api";

    let (api, mut openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .nest(
            api_prefix,
            OpenApiRouter::new()
                .nest("/apiKeys", ApiKeyApi::router())
                .nest("/auth", AuthApi::router())
                .nest("/backups", BackupApi::router())
                .nest("/bookmarks", BookmarkApi::router())
                .nest("/collections", CollectionApi::router())
                .nest("/feedEntries", FeedEntryApi::router())
                .nest("/feeds", FeedApi::router())
                .nest("/folders", FolderApi::router())
                .nest("/library", LibraryApi::router())
                // .nest("/smartFeeds", SmartFeedApi::router())
                // .with_state(SmartFeedState::new(smart_feed_service))
                .nest("/tags", TagApi::router()),
        )
        .with_state(api_state)
        .split_for_parts();

    openapi.info.title = "Colette API".to_owned();
    openapi.servers = Some(vec![Server::new(api_prefix)]);

    openapi.paths.paths = openapi
        .paths
        .paths
        .drain(RangeFull)
        .map(|(k, v)| (k.replace(&format!("{}/", api_prefix), "/"), v))
        .collect();

    let mut api = api
        .merge(Scalar::with_url(
            format!("{}/doc", api_prefix),
            openapi.clone(),
        ))
        .route(
            &format!("{}/openapi.json", api_prefix),
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

    if app_config.cors_enabled {
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
