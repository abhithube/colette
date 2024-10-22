#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("either feature \"postgres\" or feature \"sqlite\" must be enabled");

use std::{error::Error, str::FromStr, sync::Arc};

use apalis::{
    cron::{CronStream, Schedule},
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
    utils::TokioExecutor,
};
use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, feed::FeedState,
    feed_entry::FeedEntryState, profile::ProfileState, smart_feed::SmartFeedState, tag::TagState,
    Api, ApiState,
};
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::AuthService,
    backup::{BackupRepository, BackupService},
    bookmark::{BookmarkRepository, BookmarkService},
    cleanup::{CleanupRepository, CleanupService},
    feed::{FeedRepository, FeedService},
    feed_entry::{FeedEntryRepository, FeedEntryService},
    profile::{ProfileRepository, ProfileService},
    refresh::{RefreshRepository, RefreshService},
    scraper::{ScraperRepository, ScraperService},
    smart_feed::{SmartFeedRepository, SmartFeedService},
    tag::{TagRepository, TagService},
    user::UserRepository,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_session::SessionBackend;
use colette_task::{import_feeds, run_task_worker, scrape_feed, TaskQueue};
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_sessions_core::ExpiredDeletion;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_config = colette_config::load_config()?;

    #[allow(clippy::type_complexity)]
    let (
        backup_repository,
        bookmark_repository,
        cleanup_repository,
        feed_repository,
        feed_entry_repository,
        profile_repository,
        refresh_repository,
        scraper_repository,
        smart_feed_repository,
        tag_repository,
        user_repository,
        session_backend,
    ): (
        Arc<dyn BackupRepository>,
        Arc<dyn BookmarkRepository>,
        Arc<dyn CleanupRepository>,
        Arc<dyn FeedRepository>,
        Arc<dyn FeedEntryRepository>,
        Arc<dyn ProfileRepository>,
        Arc<dyn RefreshRepository>,
        Arc<dyn ScraperRepository>,
        Arc<dyn SmartFeedRepository>,
        Arc<dyn TagRepository>,
        Arc<dyn UserRepository>,
        SessionBackend,
    ) = match &app_config.database_url {
        #[cfg(feature = "postgres")]
        url if url.starts_with("postgres") => {
            let pool = sqlx::PgPool::connect(url).await?;

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
            let feed_repository =
                Arc::new(colette_postgres::PostgresFeedRepository::new(pool.clone()));
            let feed_entry_repository = Arc::new(
                colette_postgres::PostgresFeedEntryRepository::new(pool.clone()),
            );
            let profile_repository = Arc::new(colette_postgres::PostgresProfileRepository::new(
                pool.clone(),
            ));
            let refresh_repository = Arc::new(colette_postgres::PostgresRefreshRepository::new(
                pool.clone(),
            ));
            let scraper_repository = Arc::new(colette_postgres::PostgresScraperRepository::new(
                pool.clone(),
            ));
            let smart_feed_repository = Arc::new(
                colette_postgres::PostgresSmartFeedRepository::new(pool.clone()),
            );
            let tag_repository =
                Arc::new(colette_postgres::PostgresTagRepository::new(pool.clone()));
            let user_repository =
                Arc::new(colette_postgres::PostgresUserRepository::new(pool.clone()));

            let store = colette_session::PostgresStore::new(pool.clone());
            store.migrate().await?;

            (
                backup_repository,
                bookmark_repository,
                cleanup_repository,
                feed_repository,
                feed_entry_repository,
                profile_repository,
                refresh_repository,
                scraper_repository,
                smart_feed_repository,
                tag_repository,
                user_repository,
                SessionBackend::Postgres(store),
            )
        }
        #[cfg(feature = "sqlite")]
        url if url.starts_with("sqlite") => {
            let pool = sqlx::SqlitePool::connect(url).await?;

            colette_sqlite::migrate(&pool).await?;

            let backup_repository =
                Arc::new(colette_sqlite::SqliteBackupRepository::new(pool.clone()));
            let bookmark_repository =
                Arc::new(colette_sqlite::SqliteBookmarkRepository::new(pool.clone()));
            let cleanup_repository =
                Arc::new(colette_sqlite::SqliteCleanupRepository::new(pool.clone()));
            let feed_repository = Arc::new(colette_sqlite::SqliteFeedRepository::new(pool.clone()));
            let feed_entry_repository =
                Arc::new(colette_sqlite::SqliteFeedEntryRepository::new(pool.clone()));
            let profile_repository =
                Arc::new(colette_sqlite::SqliteProfileRepository::new(pool.clone()));
            let refresh_repository =
                Arc::new(colette_sqlite::SqliteRefreshRepository::new(pool.clone()));
            let scraper_repository =
                Arc::new(colette_sqlite::SqliteScraperRepository::new(pool.clone()));
            let smart_feed_repository =
                Arc::new(colette_sqlite::SqliteSmartFeedRepository::new(pool.clone()));
            let tag_repository = Arc::new(colette_sqlite::SqliteTagRepository::new(pool.clone()));
            let user_repository = Arc::new(colette_sqlite::SqliteUserRepository::new(pool.clone()));

            let store = colette_session::SqliteStore::new(pool.clone());
            store.migrate().await?;

            (
                backup_repository,
                bookmark_repository,
                cleanup_repository,
                feed_repository,
                feed_entry_repository,
                profile_repository,
                refresh_repository,
                scraper_repository,
                smart_feed_repository,
                tag_repository,
                user_repository,
                SessionBackend::Sqlite(store),
            )
        }
        _ => panic!("only PostgreSQL and SQLite are supported"),
    };

    let feed_plugin_registry = Arc::new(register_feed_plugins());

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
        Arc::new(register_bookmark_plugins()),
        base64_decoder.clone(),
    ));
    let cleanup_service = Arc::new(CleanupService::new(cleanup_repository));
    let feed_service = Arc::new(FeedService::new(
        feed_repository.clone(),
        feed_plugin_registry.clone(),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(feed_entry_repository, base64_decoder));
    let profile_service = Arc::new(ProfileService::new(profile_repository.clone()));
    let refresh_service = Arc::new(RefreshService::new(
        feed_plugin_registry.clone(),
        feed_repository.clone(),
        refresh_repository,
    ));
    let scraper_service = Arc::new(ScraperService::new(
        scraper_repository,
        feed_plugin_registry,
    ));
    let smart_feed_service = Arc::new(SmartFeedService::new(smart_feed_repository));
    let tag_service = Arc::new(TagService::new(tag_repository));

    let (scrape_feed_queue, scrape_feed_receiver) = TaskQueue::new();
    let scrape_feed_queue = Arc::new(scrape_feed_queue);

    let scrape_feed_task = ServiceBuilder::new()
        .concurrency_limit(5)
        .service(scrape_feed::Task::new(scraper_service));

    let (import_feeds_queue, import_feeds_receiver) = TaskQueue::new();
    let import_feeds_queue = Arc::new(import_feeds_queue);

    let import_feeds_task =
        ServiceBuilder::new().service(import_feeds::Task::new(scrape_feed_queue));

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(backup_service),
        BookmarkState::new(bookmark_service),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        ProfileState::new(profile_service),
        SmartFeedState::new(smart_feed_service),
        TagState::new(tag_service),
    );
    let api = Api::new(&api_state, &app_config, session_backend.clone())
        .build()
        .with_state(api_state)
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ));

    let listener = TcpListener::bind(format!("{}:{}", app_config.host, app_config.port)).await?;

    let server = async { axum::serve(listener, api).await };

    let mut monitor = Monitor::<TokioExecutor>::new().register({
        let schedule = Schedule::from_str(CRON_CLEANUP).unwrap();

        WorkerBuilder::new("cleanup")
            .data(cleanup_service)
            .stream(CronStream::new(schedule).into_stream())
            .build_fn(colette_task::cleanup)
    });

    if app_config.refresh_enabled {
        let schedule = Schedule::from_str(&app_config.cron_refresh).unwrap();

        monitor = monitor.register(
            WorkerBuilder::new("refresh-feeds")
                .data(refresh_service)
                .stream(CronStream::new(schedule).into_stream())
                .build_fn(colette_task::refresh_feeds),
        );
    }

    let _ = tokio::join!(
        server,
        monitor.run(),
        run_task_worker(scrape_feed_receiver, scrape_feed_task),
        run_task_worker(import_feeds_receiver, import_feeds_task),
        session_backend.continuously_delete_expired(tokio::time::Duration::from_secs(60))
    );

    Ok(())
}
