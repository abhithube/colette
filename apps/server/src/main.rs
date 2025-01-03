use std::{error::Error, future::Future, ops::DerefMut, pin::Pin, sync::Arc};

use axum::http::{header, HeaderValue, Method};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
    feed_entry::FeedEntryState, smart_feed::SmartFeedState, tag::TagState, Api, ApiState,
};
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, feed::FeedService,
    feed_entry::FeedEntryService, scraper::ScraperService, smart_feed::SmartFeedService,
    tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_queue::memory::InMemoryQueue;
use colette_repository::postgres::{
    PostgresBackupRepository, PostgresBookmarkRepository, PostgresFeedEntryRepository,
    PostgresFeedRepository, PostgresScraperRepository, PostgresSmartFeedRepository,
    PostgresTagRepository, PostgresUserRepository,
};
use colette_scraper::{
    bookmark::DefaultBookmarkScraper, downloader::DefaultDownloader, feed::DefaultFeedScraper,
};
use colette_session::postgres::PostgresSessionStore;
use colette_task::{import_bookmarks, import_feeds, refresh_feeds, scrape_bookmark, scrape_feed};
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use colette_worker::{run_cron_worker, run_task_worker};
use deadpool_postgres::{tokio_postgres::NoTls, Config, Runtime};
use refinery::embed_migrations;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{cookie::time::Duration, SessionManagerLayer};
use tower_sessions_core::{ExpiredDeletion, Expiry};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist"]
struct Asset;

embed_migrations!("./migrations");

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

    let mut config = Config::new();
    config.url = Some(app_config.database_url);
    let pool = config.create_pool(Some(Runtime::Tokio1), NoTls)?;

    let mut client = pool.get().await?;
    let client = client.deref_mut().deref_mut();
    migrations::runner().run_async(client).await?;

    let bookmark_repository = PostgresBookmarkRepository::new(pool.clone());
    let feed_repository = PostgresFeedRepository::new(pool.clone());

    let client = colette_http::Client::build(None, None)?;
    let downloader = DefaultDownloader::new(client.clone());
    let feed_plugin_registry = register_feed_plugins(
        client.clone(),
        downloader.clone(),
        DefaultFeedScraper::new(downloader.clone()),
    );
    let bookmark_plugin_registry = register_bookmark_plugins(
        client,
        downloader.clone(),
        DefaultBookmarkScraper::new(downloader),
    );

    let base64_encoder = Base64Encoder;

    let auth_service = Arc::new(AuthService::new(
        PostgresUserRepository::new(pool.clone()),
        ArgonHasher,
    ));
    let backup_service = BackupService::new(
        PostgresBackupRepository::new(pool.clone()),
        feed_repository.clone(),
        bookmark_repository.clone(),
        OpmlManager,
        NetscapeManager,
    );
    let bookmark_service = BookmarkService::new(
        bookmark_repository,
        bookmark_plugin_registry.clone(),
        base64_encoder.clone(),
    );
    let feed_service = Arc::new(FeedService::new(
        feed_repository,
        feed_plugin_registry.clone(),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(
        PostgresFeedEntryRepository::new(pool.clone()),
        base64_encoder,
    ));
    let scraper_service = Arc::new(ScraperService::new(
        PostgresScraperRepository::new(pool.clone()),
        feed_plugin_registry,
        bookmark_plugin_registry,
    ));
    let smart_feed_service = Arc::new(SmartFeedService::new(PostgresSmartFeedRepository::new(
        pool.clone(),
    )));
    let tag_service = Arc::new(TagService::new(PostgresTagRepository::new(pool.clone())));

    let (scrape_feed_queue, scrape_feed_receiver) = InMemoryQueue::new();
    let (scrape_bookmark_queue, scrape_bookmark_receiver) = InMemoryQueue::new();
    let (import_feeds_queue, import_feeds_receiver) = InMemoryQueue::new();
    let (import_bookmarks_queue, import_bookmarks_receiver) = InMemoryQueue::new();

    let import_feeds_queue = Arc::new(import_feeds_queue);
    let import_bookmarks_queue = Arc::new(import_bookmarks_queue);

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

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(
            Arc::new(backup_service),
            import_feeds_queue,
            import_bookmarks_queue,
        ),
        BookmarkState::new(Arc::new(bookmark_service)),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        SmartFeedState::new(smart_feed_service),
        TagState::new(tag_service),
    );

    let session_store = PostgresSessionStore::new(pool);

    let mut api = Api::new(&api_state, &app_config.api_prefix)
        .build()
        .with_state(api_state)
        .layer(
            SessionManagerLayer::new(session_store.clone())
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
        session_store.continuously_delete_expired(tokio::time::Duration::from_secs(60))
    );

    Ok(())
}
