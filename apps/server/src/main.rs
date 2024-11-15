use std::{error::Error, future::Future, pin::Pin, sync::Arc};

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

    let backup_repository = Arc::new(colette_postgres::PostgresBackupRepository::new(
        pool.clone(),
    ));
    let bookmark_repository = Arc::new(colette_postgres::PostgresBookmarkRepository::new(
        pool.clone(),
    ));
    let cleanup_repository = Arc::new(colette_postgres::PostgresCleanupRepository::new(
        pool.clone(),
    ));
    let feed_repository = Arc::new(colette_postgres::PostgresFeedRepository::new(pool.clone()));
    let feed_entry_repository = Arc::new(colette_postgres::PostgresFeedEntryRepository::new(
        pool.clone(),
    ));
    let profile_repository = Arc::new(colette_postgres::PostgresProfileRepository::new(
        pool.clone(),
    ));
    let scraper_repository = Arc::new(colette_postgres::PostgresScraperRepository::new(
        pool.clone(),
    ));
    let session_repository = colette_postgres::PostgresSessionRepository::new(pool.clone());
    let smart_feed_repository = Arc::new(colette_postgres::PostgresSmartFeedRepository::new(
        pool.clone(),
    ));
    let tag_repository = Arc::new(colette_postgres::PostgresTagRepository::new(pool.clone()));
    let user_repository = Arc::new(colette_postgres::PostgresUserRepository::new(pool.clone()));

    let feed_plugin_registry = Arc::new(register_feed_plugins());
    let bookmark_plugin_registry = Arc::new(register_bookmark_plugins());

    let base64_decoder = Arc::new(Base64Encoder);

    let auth_service = Arc::new(AuthService::new(
        user_repository,
        profile_repository.clone(),
        Arc::new(ArgonHasher),
    ));
    let backup_service = Arc::new(BackupService::new(
        backup_repository,
        feed_repository.clone(),
        bookmark_repository.clone(),
        Arc::new(OpmlManager),
        Arc::new(NetscapeManager),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository,
        bookmark_plugin_registry.clone(),
        base64_decoder.clone(),
    ));
    let cleanup_service = Arc::new(CleanupService::new(cleanup_repository));
    let feed_service = Arc::new(FeedService::new(
        feed_repository.clone(),
        feed_plugin_registry.clone(),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(feed_entry_repository, base64_decoder));
    let profile_service = Arc::new(ProfileService::new(profile_repository.clone()));
    let scraper_service = Arc::new(ScraperService::new(
        scraper_repository,
        feed_plugin_registry,
        bookmark_plugin_registry,
    ));
    let smart_feed_service = Arc::new(SmartFeedService::new(smart_feed_repository));
    let tag_service = Arc::new(TagService::new(tag_repository));

    let (scrape_feed_queue, scrape_feed_receiver) = TaskQueue::new();
    let scrape_feed_queue = Arc::new(scrape_feed_queue);

    let (scrape_bookmark_queue, scrape_bookmark_receiver) = TaskQueue::new();
    let scrape_bookmark_queue = Arc::new(scrape_bookmark_queue);

    let (import_feeds_queue, import_feeds_receiver) = TaskQueue::new();
    let import_feeds_queue = Arc::new(import_feeds_queue);

    let (import_bookmarks_queue, import_bookmarks_receiver) = TaskQueue::new();
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
    let cleanup_feeds_task = cleanup_feeds::Task::new(cleanup_service);

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(backup_service, import_feeds_queue, import_bookmarks_queue),
        BookmarkState::new(bookmark_service),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        ProfileState::new(profile_service),
        SmartFeedState::new(smart_feed_service),
        TagState::new(tag_service),
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
