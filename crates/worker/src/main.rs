use std::{error::Error, sync::Arc};

use colette_handler::*;
use colette_http::ReqwestClient;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::TokioQueue;
use colette_repository::*;
use colette_s3::S3ClientImpl;
use colette_scraper::{bookmark::BookmarkScraper, feed::FeedScraper};
use colette_smtp::{SmtpClientImpl, SmtpConfig};
use sqlx::PgPool;
use tokio::sync::Mutex;
use tower::{ServiceBuilder, ServiceExt as _};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cron_worker::CronWorker,
    job::{
        ArchiveThumbnailJobHandler, ImportBookmarksJobHandler, RefreshFeedsJobHandler,
        ScrapeBookmarkJobHandler, ScrapeFeedJobHandler,
    },
    job_worker::JobWorker,
};

mod config;
mod cron_worker;
mod job;
mod job_worker;

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

    let pool = PgPool::connect_lazy(&app_config.database.url)?;

    let bookmark_repository = PostgresBookmarkRepository::new(pool.clone());
    let collection_repository = PostgresCollectionRepository::new(pool.clone());

    let reqwest_client = reqwest::Client::builder().build()?;
    let http_client = ReqwestClient::new(reqwest_client.clone());

    let stmp_client = SmtpClientImpl::create(SmtpConfig {
        host: app_config.smtp.host,
        username: app_config.smtp.username,
        password: app_config.smtp.password,
        from_address: app_config.smtp.from_address,
    })?;

    let s3_client = S3ClientImpl::init(colette_s3::S3Config {
        access_key_id: app_config.s3.access_key_id,
        secret_access_key: app_config.s3.secret_access_key,
        region: app_config.s3.region,
        endpoint: app_config.s3.endpoint,
        bucket_name: app_config.s3.bucket_name,
        path_style_enabled: app_config.s3.path_style_enabled,
    })
    .await?;

    let (scrape_feed_producer, scrape_feed_consumer) = TokioQueue::new().split();
    let (scrape_bookmark_producer, scrape_bookmark_consumer) = TokioQueue::new().split();
    let (archive_thumbnail_producer, archive_thumbnail_consumer) = TokioQueue::new().split();
    let (import_bookmarks_producer, import_bookmarks_consumer) = TokioQueue::new().split();

    let bookmark_scraper = Arc::new(BookmarkScraper::new(
        http_client.clone(),
        register_bookmark_plugins(reqwest_client.clone()),
    ));

    let feed_repository = PostgresFeedRepository::new(pool.clone());

    let feed_scraper = Arc::new(FeedScraper::new(
        http_client.clone(),
        register_feed_plugins(reqwest_client),
    ));

    let list_bookmarks_handler = Arc::new(ListBookmarksHandler::new(
        bookmark_repository.clone(),
        collection_repository.clone(),
    ));
    let refresh_bookmark_handler = Arc::new(RefreshBookmarkHandler::new(
        bookmark_repository.clone(),
        bookmark_scraper.clone(),
    ));
    let archive_thumbnail_handler = Arc::new(ArchiveThumbnailHandler::new(
        bookmark_repository.clone(),
        http_client.clone(),
        s3_client,
    ));
    let fetch_outdated_feeds_handler =
        Arc::new(FetchOutdatedFeedsHandler::new(feed_repository.clone()));

    let mut scrape_feed_worker = JobWorker::new(
        scrape_feed_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ScrapeFeedJobHandler::new(Arc::new(
                RefreshFeedHandler::new(feed_repository, feed_scraper),
            )))
            .boxed(),
    );
    let mut scrape_bookmark_worker = JobWorker::new(
        scrape_bookmark_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ScrapeBookmarkJobHandler::new(refresh_bookmark_handler))
            .boxed(),
    );
    let mut archive_thumbnail_worker = JobWorker::new(
        archive_thumbnail_consumer,
        ServiceBuilder::new()
            .concurrency_limit(5)
            .service(ArchiveThumbnailJobHandler::new(archive_thumbnail_handler))
            .boxed(),
    );
    let mut import_bookmarks_worker = JobWorker::new(
        import_bookmarks_consumer,
        ServiceBuilder::new()
            .service(ImportBookmarksJobHandler::new(
                list_bookmarks_handler,
                Arc::new(Mutex::new(scrape_bookmark_producer)),
            ))
            .boxed(),
    );

    let start_refresh_feeds_worker = async {
        let mut worker = CronWorker::new(
            "refresh_feeds",
            "0 * * * * *".parse().unwrap(),
            ServiceBuilder::new()
                .service(RefreshFeedsJobHandler::new(
                    fetch_outdated_feeds_handler,
                    Arc::new(Mutex::new(scrape_feed_producer)),
                ))
                .boxed(),
        );

        worker.start().await;
    };

    let _ = tokio::join!(
        scrape_feed_worker.start(),
        scrape_bookmark_worker.start(),
        archive_thumbnail_worker.start(),
        import_bookmarks_worker.start(),
        start_refresh_feeds_worker
    );

    Ok(())
}
