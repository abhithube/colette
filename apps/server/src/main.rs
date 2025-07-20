use std::{error::Error, net::SocketAddr, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use chrono::Duration;
use colette_api::{ApiConfig, ApiOidcConfig, ApiServerConfig, ApiState, ApiStorageConfig};
use colette_core::{
    api_key::ApiKeyService,
    auth::{AuthConfig, AuthService, JwtConfig, OidcConfig},
    backup::BackupService,
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
    PostgresAccountRepository, PostgresApiKeyRepository, PostgresBackupRepository,
    PostgresBookmarkRepository, PostgresCollectionRepository, PostgresFeedEntryRepository,
    PostgresFeedRepository, PostgresJobRepository, PostgresStreamRepository,
    PostgresSubscriptionEntryRepository, PostgresSubscriptionRepository, PostgresTagRepository,
    PostgresUserRepository,
};
use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};
use colette_storage::{FsStorageClient, S3StorageClient, StorageAdapter};
use jsonwebtoken::{DecodingKey, EncodingKey, jwk::JwkSet};
use s3::{Bucket, Region, creds::Credentials};
use sqlx::PgPool;
use tokio::{net::TcpListener, sync::Mutex};
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker::{CronWorker, JobWorker};

use crate::config::StorageBackend;

mod config;
mod worker;

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist/"]
struct Asset;

#[derive(Debug, Clone, serde::Deserialize)]
struct OidcProviderMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
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

    let app_config = config::from_env().await?;

    let pool = {
        let pool = PgPool::connect_lazy(&app_config.database.url)?;

        {
            let mut migrator = PostgresMigrator::new(pool.clone());
            let mut runner = postgres_migrations::migrations::runner();

            if !migrator.is_fresh().await?
                && runner
                    .get_last_applied_migration_async(&mut migrator)
                    .await?
                    .is_none()
            {
                runner = runner.set_target(refinery::Target::Fake)
            }

            runner.run_async(&mut migrator).await?;
        }

        pool
    };

    let storage_adapter = match app_config.storage.backend {
        StorageBackend::Fs(ref config) => {
            StorageAdapter::Fs(FsStorageClient::new(config.path.to_owned()))
        }
        StorageBackend::S3(ref config) => {
            let mut bucket = Bucket::new(
                &config.bucket_name,
                Region::Custom {
                    region: config.region.to_owned(),
                    endpoint: config.endpoint.to_string(),
                },
                Credentials::new(
                    Some(&config.access_key_id),
                    Some(&config.secret_access_key),
                    None,
                    None,
                    None,
                )?,
            )?;
            if config.path_style_enabled {
                bucket.set_path_style();
            }

            let exists = bucket.exists().await?;
            if !exists {
                panic!("bucket does not exist with name: {}", config.bucket_name);
            }

            StorageAdapter::S3(S3StorageClient::new(bucket))
        }
    };

    let reqwest_client = reqwest::Client::builder().build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let (scrape_feed_producer, scrape_feed_consumer) = {
        let queue = LocalQueue::new().split();

        (
            JobProducerAdapter::Local(queue.0),
            JobConsumerAdapter::Local(queue.1),
        )
    };
    let (scrape_bookmark_producer, scrape_bookmark_consumer) = {
        let queue = LocalQueue::new().split();

        (
            JobProducerAdapter::Local(queue.0),
            JobConsumerAdapter::Local(queue.1),
        )
    };
    let (archive_thumbnail_producer, archive_thumbnail_consumer) = {
        let queue = LocalQueue::new().split();

        (
            JobProducerAdapter::Local(queue.0),
            JobConsumerAdapter::Local(queue.1),
        )
    };
    let (import_subscriptions_producer, import_subscriptions_consumer) = {
        let queue = LocalQueue::new().split();

        (
            JobProducerAdapter::Local(queue.0),
            JobConsumerAdapter::Local(queue.1),
        )
    };
    let (import_bookmarks_producer, import_bookmarks_consumer) = {
        let queue = LocalQueue::new().split();

        (
            JobProducerAdapter::Local(queue.0),
            JobConsumerAdapter::Local(queue.1),
        )
    };

    let bookmark_repository = Arc::new(PostgresBookmarkRepository::new(pool.clone()));
    let collection_repository = Arc::new(PostgresCollectionRepository::new(pool.clone()));
    let feed_entry_repository = Arc::new(PostgresFeedEntryRepository::new(pool.clone()));
    let job_repository = Arc::new(PostgresJobRepository::new(pool.clone()));
    let stream_repository = Arc::new(PostgresStreamRepository::new(pool.clone()));
    let subscription_repository = Arc::new(PostgresSubscriptionRepository::new(pool.clone()));
    let subscription_entry_repository =
        Arc::new(PostgresSubscriptionEntryRepository::new(pool.clone()));
    let tag_repository = Arc::new(PostgresTagRepository::new(pool.clone()));

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
        Arc::new(PostgresFeedRepository::new(pool.clone())),
        feed_entry_repository.clone(),
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
        job_repository,
        import_subscriptions_producer.clone(),
    ));

    let mut oidc_config = None::<OidcConfig>;
    if let Some(config) = app_config.oidc.clone() {
        let data = http_client.get(&config.discovery_endpoint).await?;
        let metadata = serde_json::from_slice::<OidcProviderMetadata>(&data)?;

        let data = http_client.get(&metadata.jwks_uri.parse()?).await?;
        let jwk_set = serde_json::from_slice::<JwkSet>(&data)?;

        oidc_config = Some(OidcConfig {
            client_id: config.client_id,
            issuer: metadata.issuer,
            authorization_endpoint: metadata.authorization_endpoint,
            token_endpoint: metadata.token_endpoint,
            userinfo_endpoint: metadata.userinfo_endpoint,
            redirect_uri: config.redirect_uri,
            jwk_set,
            scope: config.scope,
        })
    }

    let api_state = ApiState {
        api_key_service: Arc::new(ApiKeyService::new(Arc::new(PostgresApiKeyRepository::new(
            pool.clone(),
        )))),
        auth_service: Arc::new(AuthService::new(
            Arc::new(PostgresUserRepository::new(pool.clone())),
            Arc::new(PostgresAccountRepository::new(pool.clone())),
            http_client,
            AuthConfig {
                jwt: JwtConfig {
                    issuer: app_config.jwt.issuer,
                    audience: app_config.jwt.audience,
                    encoding_key: EncodingKey::from_secret(app_config.jwt.secret.as_bytes()),
                    decoding_key: DecodingKey::from_secret(app_config.jwt.secret.as_bytes()),
                    access_duration: Duration::minutes(15),
                    refresh_duration: Duration::days(7),
                },
                oidc: oidc_config.clone(),
            },
        )),
        backup_service: Arc::new(BackupService::new(
            Arc::new(PostgresBackupRepository::new(pool.clone())),
            bookmark_repository,
            subscription_repository,
            tag_repository.clone(),
        )),
        bookmark_service: bookmark_service.clone(),
        collection_service: Arc::new(CollectionService::new(collection_repository)),
        feed_service: feed_service.clone(),
        feed_entry_service: Arc::new(FeedEntryService::new(feed_entry_repository)),
        job_service: job_service.clone(),
        stream_service: Arc::new(StreamService::new(stream_repository.clone())),
        subscription_service: subscription_service.clone(),
        subscription_entry_service: Arc::new(SubscriptionEntryService::new(
            subscription_entry_repository,
            stream_repository,
        )),
        tag_service: Arc::new(TagService::new(tag_repository)),
        config: ApiConfig {
            server: ApiServerConfig {
                base_url: app_config.server.base_url,
            },
            oidc: app_config.oidc.map(|e| ApiOidcConfig {
                sign_in_text: e.sign_in_text,
            }),
            storage: ApiStorageConfig {
                image_base_url: app_config.storage.image_base_url,
            },
        },
    };

    let mut api = colette_api::create_router(api_state, app_config.cors.map(|e| e.origin_urls));

    if let StorageBackend::Fs(ref config) = app_config.storage.backend {
        api = api.nest_service("/uploads", ServeDir::new(&config.path))
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
        let mut worker = CronWorker::new(
            "refresh_feeds",
            "0 * * * * *".parse().unwrap(),
            job_service.clone(),
            ServiceBuilder::new()
                .service(RefreshFeedsHandler::new(
                    feed_service,
                    job_service,
                    Arc::new(Mutex::new(scrape_feed_producer)),
                ))
                .boxed(),
        );

        worker.start().await;
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

mod postgres_migrations {
    refinery::embed_migrations!("../../migrations");
}
