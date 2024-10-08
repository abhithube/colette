#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("either feature \"postgres\" or feature \"sqlite\" must be enabled");

use std::{error::Error, str::FromStr, sync::Arc};

use apalis::{
    prelude::{Monitor, WorkerBuilder, WorkerFactoryFn},
    utils::TokioExecutor,
};
use apalis_cron::{CronStream, Schedule};
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
    refresh::RefreshService, smart_feed::SmartFeedService, tag::TagService,
};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_postgres::{
    PostgresBackupRepository, PostgresBookmarkRepository, PostgresCleanupRepository,
    PostgresFeedEntryRepository, PostgresFeedRepository, PostgresProfileRepository,
    PostgresSmartFeedRepository, PostgresTagRepository, PostgresUserRepository,
};
#[cfg(feature = "postgres")]
use colette_session::PostgresStore;
use colette_session::SessionBackend;
#[cfg(feature = "sqlite")]
use colette_session::SqliteStore;
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_sessions::ExpiredDeletion;

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../web/dist"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_config = colette_config::load_config()?;

    let (
        backup_repository,
        bookmark_repository,
        cleanup_repository,
        feed_repository,
        feed_entry_repository,
        profile_repository,
        smart_feed_repository,
        tag_repository,
        user_repository,
        session_backend,
    ) = match &app_config.database_url {
        #[cfg(feature = "postgres")]
        url if url.starts_with("postgres") => {
            let pool = PgPool::connect(url).await?;

            let backup_repository = Arc::new(PostgresBackupRepository::new(pool.clone()));
            let bookmark_repository = Arc::new(PostgresBookmarkRepository::new(pool.clone()));
            let cleanup_repository = Arc::new(PostgresCleanupRepository::new(pool.clone()));
            let feed_repository = Arc::new(PostgresFeedRepository::new(pool.clone()));
            let feed_entry_repository = Arc::new(PostgresFeedEntryRepository::new(pool.clone()));
            let profile_repository = Arc::new(PostgresProfileRepository::new(pool.clone()));
            let smart_feed_repository = Arc::new(PostgresSmartFeedRepository::new(pool.clone()));
            let tag_repository = Arc::new(PostgresTagRepository::new(pool.clone()));
            let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));

            let store = PostgresStore::new(pool);
            store.migrate().await?;

            (
                backup_repository,
                bookmark_repository,
                cleanup_repository,
                feed_repository,
                feed_entry_repository,
                profile_repository,
                smart_feed_repository,
                tag_repository,
                user_repository,
                SessionBackend::Postgres(store),
            )
        }
        #[cfg(feature = "sqlite")]
        url if url.starts_with("sqlite") => {
            let pool = SqlitePool::connect(url).await?;

            let store = SqliteStore::new(pool);
            store.migrate().await?;

            SessionBackend::Sqlite(store)
        }
        _ => panic!("only PostgreSQL and SQLite are supported"),
    };

    let deletion_task = tokio::task::spawn(
        session_backend
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

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
        feed_plugin_registry,
        feed_repository.clone(),
        profile_repository.clone(),
    ));
    let smart_feed_service = Arc::new(SmartFeedService::new(smart_feed_repository));
    let tag_service = Arc::new(TagService::new(tag_repository));

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
    let api = Api::new(&api_state, &app_config, session_backend)
        .build()
        .with_state(api_state)
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ));

    let listener = TcpListener::bind(format!("{}:{}", app_config.host, app_config.port)).await?;

    let server = async { axum::serve(listener, api).await };

    let refresh_worker = async {
        if app_config.refresh_enabled {
            let schedule = Schedule::from_str(&app_config.cron_refresh).unwrap();

            let worker = WorkerBuilder::new("refresh-feeds")
                .data(refresh_service)
                .backend(CronStream::new(schedule))
                .build_fn(colette_task::refresh_feeds);

            Monitor::<TokioExecutor>::new().register(worker).run().await
        } else {
            Ok(())
        }
    };
    let cleanup_worker = async {
        let schedule = Schedule::from_str(CRON_CLEANUP).unwrap();

        let worker = WorkerBuilder::new("cleanup")
            .data(cleanup_service)
            .backend(CronStream::new(schedule))
            .build_fn(colette_task::cleanup);

        Monitor::<TokioExecutor>::new().register(worker).run().await
    };

    let _ = tokio::join!(server, refresh_worker, cleanup_worker);

    deletion_task.await??;

    Ok(())
}
