use std::{error::Error, net::SocketAddr, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::ApiState;
use colette_auth::AuthAdapter;
use colette_core::{
    api_key::ApiKeyService, backup::BackupService, bookmark::BookmarkService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    job::JobService, stream::StreamService, subscription::SubscriptionService,
    subscription_entry::SubscriptionEntryService, tag::TagService,
};
use colette_http::ReqwestClient;
use colette_migration::PostgresMigrator;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::{JobConsumerAdapter, JobProducerAdapter, LocalQueue};
use colette_repository::postgres::{
    PostgresApiKeyRepository, PostgresBackupRepository, PostgresBookmarkRepository,
    PostgresCollectionRepository, PostgresFeedEntryRepository, PostgresFeedRepository,
    PostgresJobRepository, PostgresStreamRepository, PostgresSubscriptionEntryRepository,
    PostgresSubscriptionRepository, PostgresTagRepository,
};
use colette_storage::{LocalStorageClient, StorageAdapter};
use config::{QueueConfig, StorageConfig};
use deadpool_postgres::{Manager, ManagerConfig, Pool};
use job::{
    JobWorker, archive_thumbnail::ArchiveThumbnailHandler,
    import_bookmarks::ImportBookmarksHandler, import_feeds::ImportFeedsHandler,
    refresh_feeds::RefreshFeedsHandler, scrape_bookmark::ScrapeBookmarkHandler,
    scrape_feed::ScrapeFeedHandler,
};
use refinery::embed_migrations;
use tokio::{net::TcpListener, sync::Mutex};
use tokio_postgres::NoTls;
use torii::Torii;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod job;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

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

    let manager = Manager::from_config(
        app_config.database_url.parse()?,
        NoTls,
        ManagerConfig::default(),
    );

    let pool = Pool::builder(manager).build()?;

    let mut migrator = PostgresMigrator::new(pool.clone());
    migrations::runner().run_async(&mut migrator).await?;

    let (storage_adapter, image_base_url) = match app_config.storage.clone() {
        StorageConfig::Local(config) => (
            StorageAdapter::Local(LocalStorageClient::new(config.path)),
            format!("http://0.0.0.0:{}/uploads/", app_config.server.port)
                .parse()
                .unwrap(),
        ), // StorageConfig::S3(config) => {
           //     let s3 = AmazonS3Builder::new()
           //         .with_access_key_id(config.access_key_id)
           //         .with_secret_access_key(config.secret_access_key)
           //         .with_region(config.region)
           //         .with_endpoint(config.endpoint.origin().ascii_serialization())
           //         .with_bucket_name(&config.bucket_name)
           //         .with_allow_http(true)
           //         .build()?;

           //     let base_url = config
           //         .endpoint
           //         .join(&format!("{}/", config.bucket_name))
           //         .unwrap();

           //     (StorageAdapter::S3(s3), base_url)
           // }
    };

    let reqwest_client = reqwest::Client::builder().https_only(true).build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let (scrape_feed_producer, scrape_feed_consumer) = match &app_config.queue {
        QueueConfig::Local => {
            let queue = LocalQueue::new().split();

            (
                JobProducerAdapter::Local(queue.0),
                JobConsumerAdapter::Local(queue.1),
            )
        }
    };
    let (scrape_bookmark_producer, scrape_bookmark_consumer) = match &app_config.queue {
        QueueConfig::Local => {
            let queue = LocalQueue::new().split();

            (
                JobProducerAdapter::Local(queue.0),
                JobConsumerAdapter::Local(queue.1),
            )
        }
    };
    let (archive_thumbnail_producer, archive_thumbnail_consumer) = match &app_config.queue {
        QueueConfig::Local => {
            let queue = LocalQueue::new().split();

            (
                JobProducerAdapter::Local(queue.0),
                JobConsumerAdapter::Local(queue.1),
            )
        }
    };
    let (import_feeds_producer, import_feeds_consumer) = match &app_config.queue {
        QueueConfig::Local => {
            let queue = LocalQueue::new().split();

            (
                JobProducerAdapter::Local(queue.0),
                JobConsumerAdapter::Local(queue.1),
            )
        }
    };
    let (import_bookmarks_producer, import_bookmarks_consumer) = match &app_config.queue {
        QueueConfig::Local => {
            let queue = LocalQueue::new().split();

            (
                JobProducerAdapter::Local(queue.0),
                JobConsumerAdapter::Local(queue.1),
            )
        }
    };

    let bookmark_repository = PostgresBookmarkRepository::new(pool.clone());
    let collection_repository = PostgresCollectionRepository::new(pool.clone());
    let feed_repository = PostgresFeedRepository::new(pool.clone());
    let job_repository = PostgresJobRepository::new(pool.clone());
    let stream_repository = PostgresStreamRepository::new(pool.clone());
    let subscription_repository = PostgresSubscriptionRepository::new(pool.clone());
    let subscription_entry_repository = PostgresSubscriptionEntryRepository::new(pool.clone());
    let tag_repository = PostgresTagRepository::new(pool.clone());

    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository.clone(),
        tag_repository.clone(),
        collection_repository.clone(),
        job_repository.clone(),
        http_client.clone(),
        storage_adapter,
        archive_thumbnail_producer.clone(),
        register_bookmark_plugins(reqwest_client.clone()),
    ));
    let feed_service = Arc::new(FeedService::new(
        feed_repository.clone(),
        http_client.clone(),
        register_feed_plugins(reqwest_client),
    ));
    let job_service = Arc::new(JobService::new(job_repository.clone()));
    let subscription_service = Arc::new(SubscriptionService::new(
        subscription_repository.clone(),
        tag_repository.clone(),
        subscription_entry_repository.clone(),
    ));

    if let Some(config) = app_config.cron {
        // let schedule = config.schedule.parse()?;

        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(RefreshFeedsHandler::new(
                feed_service.clone(),
                job_service.clone(),
                Arc::new(Mutex::new(scrape_feed_producer.clone())),
            ))
            .boxed();
    }

    let api_state = ApiState {
        auth: Arc::new(
            Torii::new(Arc::new(AuthAdapter::Postgres(
                colette_auth::PostgresBackend::new(pool.clone()),
            )))
            .with_password_plugin(),
        ),
        api_key_service: Arc::new(ApiKeyService::new(PostgresApiKeyRepository::new(
            pool.clone(),
        ))),
        backup_service: Arc::new(BackupService::new(
            PostgresBackupRepository::new(pool.clone()),
            subscription_repository,
            bookmark_repository,
            job_repository,
            import_feeds_producer,
            import_bookmarks_producer,
        )),
        bookmark_service: bookmark_service.clone(),
        collection_service: Arc::new(CollectionService::new(collection_repository)),
        feed_service: feed_service.clone(),
        feed_entry_service: Arc::new(FeedEntryService::new(PostgresFeedEntryRepository::new(
            pool,
        ))),
        job_service: job_service.clone(),
        stream_service: Arc::new(StreamService::new(stream_repository.clone())),
        subscription_service: subscription_service.clone(),
        subscription_entry_service: Arc::new(SubscriptionEntryService::new(
            subscription_entry_repository,
            stream_repository,
        )),
        tag_service: Arc::new(TagService::new(tag_repository)),
        image_base_url,
    };

    let mut api = colette_api::create_router(api_state, app_config.cors.map(|e| e.origin_urls));

    if let StorageConfig::Local(config) = app_config.storage {
        api = api.nest_service("/uploads", ServeDir::new(config.path))
    }

    api = api.fallback_service(ServeEmbed::<Asset>::with_parameters(
        Some(String::from("index.html")),
        FallbackBehavior::Ok,
        None,
    ));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", app_config.server.port)).await?;
    let server = axum::serve(
        listener,
        api.into_make_service_with_connect_info::<SocketAddr>(),
    );

    let mut scrape_feed_worker = JobWorker::new(
        job_service.clone(),
        scrape_feed_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ScrapeFeedHandler::new(feed_service))
            .boxed(),
    );
    let mut scrape_bookmark_worker = JobWorker::new(
        job_service.clone(),
        scrape_bookmark_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ScrapeBookmarkHandler::new(bookmark_service.clone()))
            .boxed(),
    );
    let mut archive_thumbnail_worker = JobWorker::new(
        job_service.clone(),
        archive_thumbnail_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ArchiveThumbnailHandler::new(bookmark_service.clone()))
            .boxed(),
    );
    let mut import_feeds_worker = JobWorker::new(
        job_service.clone(),
        import_feeds_consumer,
        ServiceBuilder::new()
            .service(ImportFeedsHandler::new(
                subscription_service,
                job_service.clone(),
                Arc::new(Mutex::new(scrape_feed_producer)),
            ))
            .boxed(),
    );
    let mut import_bookmarks_worker = JobWorker::new(
        job_service.clone(),
        import_bookmarks_consumer,
        ServiceBuilder::new()
            .service(ImportBookmarksHandler::new(
                bookmark_service,
                job_service,
                Arc::new(Mutex::new(scrape_bookmark_producer)),
            ))
            .boxed(),
    );

    let _ = tokio::join!(
        server,
        scrape_feed_worker.start(),
        scrape_bookmark_worker.start(),
        archive_thumbnail_worker.start(),
        import_feeds_worker.start(),
        import_bookmarks_worker.start()
    );

    Ok(())
}
