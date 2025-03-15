use std::{error::Error, ops::RangeFull, str::FromStr, sync::Arc, time::Duration};

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
};
use apalis_cron::CronStream;
use api::{
    ApiState,
    api_key::ApiKeyApi,
    auth::AuthApi,
    backup::BackupApi,
    bookmark::BookmarkApi,
    collection::CollectionApi,
    common::{BaseError, BooleanOp, DateOp, TextOp},
    feed::FeedApi,
    feed_entry::FeedEntryApi,
    stream::StreamApi,
    subscription::SubscriptionApi,
    subscription_entry::SubscriptionEntryApi,
    tag::TagApi,
};
use axum::{
    http::{HeaderValue, Method, header},
    routing,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_core::{
    api_key::ApiKeyService, auth::AuthService, backup::BackupService, bookmark::BookmarkService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    stream::StreamService, subscription::SubscriptionService,
    subscription_entry::SubscriptionEntryService, tag::TagService,
};
use colette_http::ReqwestClient;
use colette_job::SqliteStorage;
use colette_migration::SqliteMigrator;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_session::{RedisStore, SessionAdapter};
use colette_storage::StorageAdapter;
use config::{DatabaseConfig, JobConfig, SessionConfig, StorageConfig};
use job::{
    archive_thumbnail, import_bookmarks, import_feeds, refresh_feeds, scrape_bookmark, scrape_feed,
};
use object_store::{aws::AmazonS3Builder, local::LocalFileSystem};
use refinery::embed_migrations;
use repository::{
    api_key::SqliteApiKeyRepository, backup::SqliteBackupRepository,
    bookmark::SqliteBookmarkRepository, collection::SqliteCollectionRepository,
    feed::SqliteFeedRepository, feed_entry::SqliteFeedEntryRepository,
    stream::SqliteStreamRepository, subscription::SqliteSubscriptionRepository,
    subscription_entry::SqliteSubscriptionEntryRepository, tag::SqliteTagRepository,
    user::SqliteUserRepository,
};
use sqlx::{
    Pool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tower_sessions::{SessionManagerLayer, cookie};
use tower_sessions_core::{Expiry, session_store::ExpiredDeletion};
use tower_sessions_sqlx_store::SqliteStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, openapi::Server};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

mod api;
mod config;
mod job;
mod repository;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(BaseError, TextOp, BooleanOp, DateOp)))]
struct ApiDoc;

embed_migrations!("../../migrations");

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

    let app_config = config::from_env()?;

    let pool = match app_config.database {
        DatabaseConfig::Sqlite(config) => {
            let options = SqliteConnectOptions::from_str(config.url.to_str().unwrap())?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal);

            let pool = Pool::connect_with(options.journal_mode(SqliteJournalMode::Wal)).await?;

            let mut migrator = SqliteMigrator::new(pool.clone());
            migrations::runner().run_async(&mut migrator).await?;

            pool
        }
    };

    let (session_adapter, deletion_task) = match app_config.session {
        SessionConfig::Sqlite(config) => {
            let options = SqliteConnectOptions::from_str(config.url.to_str().unwrap())?
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal);

            let pool = Pool::connect_with(options.journal_mode(SqliteJournalMode::Wal)).await?;

            let store = SqliteStore::new(pool);
            store.migrate().await?;

            (
                SessionAdapter::Sqlite(store.clone()),
                Some(tokio::task::spawn(
                    store.continuously_delete_expired(Duration::from_secs(60)),
                )),
            )
        }
        SessionConfig::Redis(config) => {
            let redis = redis::Client::open(config.url)?;
            let conn = redis.get_multiplexed_async_connection().await?;
            (SessionAdapter::Redis(RedisStore::new(conn)), None)
        }
    };

    let job_pool = match app_config.job {
        JobConfig::Sqlite(config) => {
            let options = SqliteConnectOptions::from_str(config.url.to_str().unwrap())?
                .create_if_missing(true);

            let pool = Pool::connect_with(options.journal_mode(SqliteJournalMode::Wal)).await?;

            SqliteStorage::setup(&pool).await?;

            pool
        }
    };

    let (storage_adapter, image_base_url) = match app_config.storage.clone() {
        StorageConfig::Fs(config) => {
            let fs = LocalFileSystem::new_with_prefix(config.path)?.with_automatic_cleanup(true);

            (
                StorageAdapter::Local(Arc::new(fs)),
                format!("http://0.0.0.0:{}/uploads/", app_config.server.port)
                    .parse()
                    .unwrap(),
            )
        }
        StorageConfig::S3(config) => {
            let s3 = AmazonS3Builder::new()
                .with_access_key_id(config.access_key_id)
                .with_secret_access_key(config.secret_access_key)
                .with_region(config.region)
                .with_endpoint(config.endpoint.origin().ascii_serialization())
                .with_bucket_name(&config.bucket_name)
                .with_allow_http(true)
                .build()?;

            let base_url = config
                .endpoint
                .join(&format!("{}/", config.bucket_name))
                .unwrap();

            (StorageAdapter::S3(s3), base_url)
        }
    };

    let reqwest_client = reqwest::Client::builder().https_only(true).build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let scrape_feed_storage = SqliteStorage::new(job_pool.clone());
    let scrape_bookmark_storage = SqliteStorage::new(job_pool.clone());
    let archive_thumbnail_storage = SqliteStorage::new(job_pool.clone());
    let import_feeds_storage = SqliteStorage::new(job_pool.clone());
    let import_bookmarks_storage = SqliteStorage::new(job_pool);

    let bookmark_repository = SqliteBookmarkRepository::new(pool.clone());
    let collection_repository = SqliteCollectionRepository::new(pool.clone());
    let feed_repository = SqliteFeedRepository::new(pool.clone());
    let stream_repository = SqliteStreamRepository::new(pool.clone());
    let subscription_repository = SqliteSubscriptionRepository::new(pool.clone());
    let subscription_entry_repository = SqliteSubscriptionEntryRepository::new(pool.clone());
    let tag_repository = SqliteTagRepository::new(pool.clone());

    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository.clone(),
        tag_repository.clone(),
        collection_repository.clone(),
        http_client.clone(),
        storage_adapter,
        Arc::new(Mutex::new(archive_thumbnail_storage.clone())),
        register_bookmark_plugins(reqwest_client.clone()),
    ));
    let feed_service = Arc::new(FeedService::new(
        feed_repository.clone(),
        http_client.clone(),
        register_feed_plugins(reqwest_client),
    ));

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

    let archive_thumbnail_worker = WorkerBuilder::new("archive-thumbnail")
        .enable_tracing()
        .concurrency(5)
        .data(bookmark_service.clone())
        .backend(archive_thumbnail_storage)
        .build_fn(archive_thumbnail::run);

    let import_feeds_worker = WorkerBuilder::new("import-feeds")
        .enable_tracing()
        .data(scrape_feed_storage.clone())
        .backend(import_feeds_storage.clone())
        .build_fn(import_feeds::run);

    let import_bookmarks_worker = WorkerBuilder::new("import-bookmarks")
        .enable_tracing()
        .data(scrape_bookmark_storage)
        .backend(import_bookmarks_storage.clone())
        .build_fn(import_bookmarks::run);

    let mut monitor = Monitor::new()
        .register(scrape_feed_worker)
        .register(scrape_bookmark_worker)
        .register(archive_thumbnail_worker)
        .register(import_feeds_worker)
        .register(import_bookmarks_worker);

    if let Some(config) = app_config.cron {
        let schedule = config.schedule.parse()?;

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

    let api_state = ApiState {
        api_key_service: Arc::new(ApiKeyService::new(SqliteApiKeyRepository::new(
            pool.clone(),
        ))),
        auth_service: Arc::new(AuthService::new(SqliteUserRepository::new(pool.clone()))),
        backup_service: Arc::new(BackupService::new(
            SqliteBackupRepository::new(pool.clone()),
            subscription_repository.clone(),
            bookmark_repository,
            Arc::new(Mutex::new(import_feeds_storage)),
            Arc::new(Mutex::new(import_bookmarks_storage)),
        )),
        bookmark_service,
        collection_service: Arc::new(CollectionService::new(collection_repository)),
        feed_service,
        feed_entry_service: Arc::new(FeedEntryService::new(SqliteFeedEntryRepository::new(pool))),
        stream_service: Arc::new(StreamService::new(stream_repository.clone())),
        subscription_service: Arc::new(SubscriptionService::new(
            subscription_repository,
            tag_repository.clone(),
            subscription_entry_repository.clone(),
        )),
        subscription_entry_service: Arc::new(SubscriptionEntryService::new(
            subscription_entry_repository,
            stream_repository,
        )),
        tag_service: Arc::new(TagService::new(tag_repository)),
        image_base_url,
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
                .nest("/streams", StreamApi::router())
                .nest("/subscriptionEntries", SubscriptionEntryApi::router())
                .nest("/subscriptions", SubscriptionApi::router())
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
            SessionManagerLayer::new(session_adapter.clone())
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(cookie::time::Duration::days(1))),
        )
        .layer(TraceLayer::new_for_http())
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ));

    if let StorageConfig::Fs(config) = app_config.storage {
        api = api.nest_service("/uploads", ServeDir::new(config.path))
    }

    if let Some(config) = app_config.cors {
        let origins = config
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

    let listener = TcpListener::bind(format!("0.0.0.0:{}", app_config.server.port)).await?;
    let server = axum::serve(listener, api);

    let _ = tokio::join!(monitor, server);

    if let Some(deletion_task) = deletion_task {
        deletion_task.await??;
    }

    Ok(())
}
