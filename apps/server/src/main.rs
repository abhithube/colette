use std::{error::Error, net::SocketAddr, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{ApiConfig, ApiOidcConfig, ApiState, ApiStorageConfig};
use colette_core::{
    api_key::ApiKeyService,
    auth::{AuthService, OidcConfig},
    bookmark::BookmarkService,
    collection::CollectionService,
    feed::FeedService,
    feed_entry::FeedEntryService,
    job::JobService,
    stream::StreamService,
    subscription::SubscriptionService,
    subscription_entry::SubscriptionEntryService,
    tag::TagService,
};
use colette_http::{HttpClient, ReqwestClient};
use colette_job::{
    archive_thumbnail::ArchiveThumbnailHandler, import_bookmarks::ImportBookmarksHandler,
    import_subscriptions::ImportSubscriptionsHandler, refresh_feeds::RefreshFeedsHandler,
    scrape_bookmark::ScrapeBookmarkHandler, scrape_feed::ScrapeFeedHandler,
};
use colette_migration::PostgresMigrator;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::{JobConsumerAdapter, JobProducerAdapter, LocalQueue};
use colette_repository::postgres::{
    PostgresApiKeyRepository, PostgresBookmarkRepository, PostgresCollectionRepository,
    PostgresFeedEntryRepository, PostgresFeedRepository, PostgresJobRepository,
    PostgresStreamRepository, PostgresSubscriptionEntryRepository, PostgresSubscriptionRepository,
    PostgresTagRepository, PostgresUserRepository,
};
use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};
use colette_storage::{FsStorageClient, S3StorageClient, StorageAdapter};
use config::{QueueConfig, S3RequestStyle, StorageConfig};
use deadpool_postgres::{Manager, ManagerConfig, Pool};
use jsonwebtoken::jwk::JwkSet;
use refinery::embed_migrations;
use s3::{Bucket, Region, creds::Credentials};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_postgres::NoTls;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker::{CronWorker, JobWorker};

mod config;
mod worker;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

embed_migrations!("../../migrations");

#[derive(Debug, Clone, serde::Deserialize)]
struct OidcProviderMetadata {
    issuer: String,
    userinfo_endpoint: String,
    jwks_uri: String,
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
        StorageConfig::Fs(config) => (
            StorageAdapter::Fs(FsStorageClient::new(config.path)),
            format!("http://0.0.0.0:{}/uploads/", app_config.server.port)
                .parse()
                .unwrap(),
        ),
        StorageConfig::S3(config) => {
            let mut base_url = config.endpoint;

            let mut bucket = Bucket::new(
                &config.bucket_name,
                Region::Custom {
                    region: config.region,
                    endpoint: base_url.origin().ascii_serialization(),
                },
                Credentials::new(
                    Some(&config.access_key_id),
                    Some(&config.secret_access_key),
                    None,
                    None,
                    None,
                )?,
            )?;
            match config.path_style {
                S3RequestStyle::Path => {
                    base_url.set_path(&format!("{}/", bucket.name));
                    bucket.set_path_style();
                }
                S3RequestStyle::VirtualHost => {
                    base_url.set_host(Some(&format!(
                        "{}.{}",
                        bucket.name,
                        base_url.host_str().unwrap()
                    )))?;
                }
            }

            let exists = bucket.exists().await?;
            if !exists {
                panic!("bucket does not exist with name: {}", config.bucket_name);
            }

            (StorageAdapter::S3(S3StorageClient::new(bucket)), base_url)
        }
    };

    let reqwest_client = reqwest::Client::builder().build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let oidc_config = {
        let data = http_client.get(&app_config.oidc.discovery_endpoint).await?;
        let metadata = serde_json::from_slice::<OidcProviderMetadata>(&data)?;

        let data = http_client.get(&metadata.jwks_uri.parse()?).await?;
        let jwk_set = serde_json::from_slice::<JwkSet>(&data)?;

        OidcConfig {
            client_id: app_config.oidc.client_id.clone(),
            issuer: metadata.issuer,
            userinfo_endpoint: metadata.userinfo_endpoint,
            jwk_set,
        }
    };

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
    let (import_subscriptions_producer, import_subscriptions_consumer) = match &app_config.queue {
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
        BookmarkScraper::new(
            http_client.clone(),
            register_bookmark_plugins(reqwest_client.clone()),
        ),
        storage_adapter,
        archive_thumbnail_producer.clone(),
        import_bookmarks_producer.clone(),
    ));
    let feed_service = Arc::new(FeedService::new(
        feed_repository.clone(),
        http_client.clone(),
        FeedScraper::new(
            http_client.clone(),
            register_feed_plugins(reqwest_client.clone()),
        ),
    ));
    let job_service = Arc::new(JobService::new(job_repository.clone()));
    let subscription_service = Arc::new(SubscriptionService::new(
        subscription_repository.clone(),
        tag_repository.clone(),
        subscription_entry_repository.clone(),
        job_repository.clone(),
        import_subscriptions_producer.clone(),
    ));

    let api_state = ApiState {
        api_key_service: Arc::new(ApiKeyService::new(PostgresApiKeyRepository::new(
            pool.clone(),
        ))),
        auth_service: Arc::new(AuthService::new(
            PostgresUserRepository::new(pool.clone()),
            http_client,
            oidc_config.clone(),
        )),
        bookmark_service: bookmark_service.clone(),
        collection_service: Arc::new(CollectionService::new(collection_repository)),
        feed_service: feed_service.clone(),
        feed_entry_service: Arc::new(FeedEntryService::new(PostgresFeedEntryRepository::new(
            pool.clone(),
        ))),
        job_service: job_service.clone(),
        stream_service: Arc::new(StreamService::new(stream_repository.clone())),
        subscription_service: subscription_service.clone(),
        subscription_entry_service: Arc::new(SubscriptionEntryService::new(
            subscription_entry_repository,
            stream_repository,
        )),
        tag_service: Arc::new(TagService::new(tag_repository)),
        config: ApiConfig {
            oidc: ApiOidcConfig {
                client_id: oidc_config.client_id,
                redirect_uri: app_config.oidc.redirect_uri.into(),
                issuer: oidc_config.issuer,
            },
            storage: ApiStorageConfig {
                base_url: image_base_url,
            },
        },
    };

    let mut api = colette_api::create_router(api_state, app_config.cors.map(|e| e.origin_urls));

    if let StorageConfig::Fs(config) = app_config.storage {
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
            .service(ScrapeFeedHandler::new(feed_service.clone()))
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
    let mut import_subscriptions_worker = JobWorker::new(
        job_service.clone(),
        import_subscriptions_consumer,
        ServiceBuilder::new()
            .service(ImportSubscriptionsHandler::new(
                subscription_service,
                job_service.clone(),
                Arc::new(Mutex::new(scrape_feed_producer.clone())),
            ))
            .boxed(),
    );
    let mut import_bookmarks_worker = JobWorker::new(
        job_service.clone(),
        import_bookmarks_consumer,
        ServiceBuilder::new()
            .service(ImportBookmarksHandler::new(
                bookmark_service,
                job_service.clone(),
                Arc::new(Mutex::new(scrape_bookmark_producer)),
            ))
            .boxed(),
    );

    let start_refresh_feeds_worker = async {
        if let Some(config) = app_config.cron {
            let schedule = config.schedule.parse().unwrap();

            let mut worker = CronWorker::new(
                schedule,
                job_service.clone(),
                ServiceBuilder::new()
                    .concurrency_limit(5)
                    .service(RefreshFeedsHandler::new(
                        feed_service,
                        job_service,
                        Arc::new(Mutex::new(scrape_feed_producer)),
                    ))
                    .boxed(),
            );

            worker.start().await;
        }
    };

    let _ = tokio::join!(
        server,
        scrape_feed_worker.start(),
        scrape_bookmark_worker.start(),
        archive_thumbnail_worker.start(),
        import_subscriptions_worker.start(),
        import_bookmarks_worker.start(),
        start_refresh_feeds_worker
    );

    Ok(())
}
