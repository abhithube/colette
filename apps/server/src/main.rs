use std::{error::Error, future::Future, pin::Pin};

use axum::http::{header, HeaderValue, Method};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
    feed_entry::FeedEntryState, profile::ProfileState, smart_feed::SmartFeedState, tag::TagState,
    Api, ApiState,
};
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, cleanup::CleanupService,
    feed::FeedService, feed_entry::FeedEntryService, profile::ProfileService,
    scraper::ScraperService, smart_feed::SmartFeedService, tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_scraper::{DefaultBookmarkScraper, DefaultDownloader, DefaultFeedScraper};
use colette_task::{
    cleanup_feeds, import_bookmarks, import_feeds, refresh_feeds, run_cron_worker, run_task_worker,
    scrape_bookmark, scrape_feed, TaskQueue,
};
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{cookie::time::Duration, SessionManagerLayer};
use tower_sessions_core::{ExpiredDeletion, Expiry};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist"]
struct Asset;

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

    let app_config = colette_config::load_config()?;

    let pool = sqlx::PgPool::connect(&app_config.database_url).await?;

    colette_postgres::migrate(&pool).await?;

    let backup_repository = Box::new(colette_postgres::PostgresBackupRepository::new(
        pool.clone(),
    ));
    let bookmark_repository = Box::new(colette_postgres::PostgresBookmarkRepository::new(
        pool.clone(),
    ));
    let cleanup_repository = Box::new(colette_postgres::PostgresCleanupRepository::new(
        pool.clone(),
    ));
    let feed_repository = Box::new(colette_postgres::PostgresFeedRepository::new(pool.clone()));
    let feed_entry_repository = Box::new(colette_postgres::PostgresFeedEntryRepository::new(
        pool.clone(),
    ));
    let profile_repository = Box::new(colette_postgres::PostgresProfileRepository::new(
        pool.clone(),
    ));
    let scraper_repository = Box::new(colette_postgres::PostgresScraperRepository::new(
        pool.clone(),
    ));
    let session_repository = colette_postgres::PostgresSessionRepository::new(pool.clone());
    let smart_feed_repository = Box::new(colette_postgres::PostgresSmartFeedRepository::new(
        pool.clone(),
    ));
    let tag_repository = Box::new(colette_postgres::PostgresTagRepository::new(pool.clone()));
    let user_repository = Box::new(colette_postgres::PostgresUserRepository::new(pool.clone()));

    let client = reqwest::Client::new();
    let downloader = Box::new(DefaultDownloader::new(client.clone()));
    let feed_scraper = Box::new(DefaultFeedScraper::new(downloader.clone()));
    let bookmark_scraper = Box::new(DefaultBookmarkScraper::new(downloader.clone()));
    let feed_plugin_registry = Box::new(register_feed_plugins(downloader.clone(), feed_scraper));
    let bookmark_plugin_registry = Box::new(register_bookmark_plugins(client, bookmark_scraper));

    let base64_decoder = Box::new(Base64Encoder);

    let feed_service = FeedService::new(feed_repository.clone(), feed_plugin_registry.clone());
    let scraper_service = ScraperService::new(
        scraper_repository,
        feed_plugin_registry,
        bookmark_plugin_registry.clone(),
    );

    let (scrape_feed_queue, scrape_feed_receiver) = TaskQueue::new();
    let (scrape_bookmark_queue, scrape_bookmark_receiver) = TaskQueue::new();
    let (import_feeds_queue, import_feeds_receiver) = TaskQueue::new();
    let (import_bookmarks_queue, import_bookmarks_receiver) = TaskQueue::new();

    let scrape_feed_task = ServiceBuilder::new()
        .concurrency_limit(5)
        .service(scrape_feed::Task::new(scraper_service.clone()));
    let scrape_bookmark_task = ServiceBuilder::new()
        .concurrency_limit(5)
        .service(scrape_bookmark::Task::new(scraper_service));
    let refresh_feeds_task =
        refresh_feeds::Task::new(feed_service.clone(), scrape_feed_queue.clone());
    let import_feeds_task = import_feeds::Task::new(scrape_feed_queue);
    let import_bookmarks_task = import_bookmarks::Task::new(scrape_bookmark_queue);
    let cleanup_feeds_task = cleanup_feeds::Task::new(CleanupService::new(cleanup_repository));

    let api_state = ApiState::new(
        AuthState::new(AuthService::new(
            user_repository,
            profile_repository.clone(),
            Box::new(ArgonHasher),
        )),
        BackupState::new(
            BackupService::new(
                backup_repository,
                feed_repository.clone(),
                bookmark_repository.clone(),
                Box::new(OpmlManager),
                Box::new(NetscapeManager),
            ),
            import_feeds_queue,
            import_bookmarks_queue,
        ),
        BookmarkState::new(BookmarkService::new(
            bookmark_repository,
            bookmark_plugin_registry,
            base64_decoder.clone(),
        )),
        FeedState::new(feed_service),
        FeedEntryState::new(FeedEntryService::new(feed_entry_repository, base64_decoder)),
        ProfileState::new(ProfileService::new(profile_repository)),
        SmartFeedState::new(SmartFeedService::new(smart_feed_repository)),
        TagState::new(TagService::new(tag_repository)),
    );
    let mut api = Api::new(&api_state)
        .build()
        .with_state(api_state)
        .layer(
            SessionManagerLayer::new(session_repository.clone())
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
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

    let server = async { axum::serve(listener, api).await };

    let refresh_task_worker = if app_config.refresh_enabled {
        Box::pin(run_cron_worker(
            &app_config.cron_refresh,
            refresh_feeds_task,
        ))
    } else {
        Box::pin(std::future::ready(())) as Pin<Box<dyn Future<Output = ()> + Send>>
    };

    let _ = tokio::join!(
        server,
        run_task_worker(scrape_feed_receiver, scrape_feed_task),
        run_task_worker(scrape_bookmark_receiver, scrape_bookmark_task),
        run_task_worker(import_feeds_receiver, import_feeds_task),
        run_task_worker(import_bookmarks_receiver, import_bookmarks_task),
        refresh_task_worker,
        run_cron_worker("0 0 0 * * *", cleanup_feeds_task),
        session_repository.continuously_delete_expired(tokio::time::Duration::from_secs(60))
    );

    Ok(())
}
