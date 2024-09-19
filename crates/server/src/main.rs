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
    auth::AuthState, backup::BackupState, bookmark::BookmarkState, collection::CollectionState,
    feed::FeedState, feed_entry::FeedEntryState, profile::ProfileState, tag::TagState, Api,
    ApiState,
};
use colette_backup::{netscape::NetscapeManager, opml::OpmlManager};
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService, cleanup::CleanupService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    profile::ProfileService, refresh::RefreshService, tag::TagService,
};
use colette_migration::{Migrator, MigratorTrait};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_repository::{
    BackupSqlRepository, BookmarkSqlRepository, CleanupSqlRepository, CollectionSqlRepository,
    FeedEntrySqlRepository, FeedSqlRepository, ProfileSqlRepository, TagSqlRepository,
    UserSqlRepository,
};
#[cfg(feature = "postgres")]
use colette_session::PostgresStore;
use colette_session::SessionBackend;
#[cfg(feature = "sqlite")]
use colette_session::SqliteStore;
use colette_util::{base64::Base64Encoder, password::ArgonHasher};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend};
use tokio::net::TcpListener;
use tower_sessions::ExpiredDeletion;

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../packages/web/dist"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_config = colette_config::load_config()?;

    let mut opts = ConnectOptions::new(&app_config.database_url);
    opts.max_connections(100);

    let db = Database::connect(opts).await?;
    Migrator::up(&db, None).await?;

    let session_backend = match db.get_database_backend() {
        #[cfg(feature = "postgres")]
        DatabaseBackend::Postgres => {
            let store = PostgresStore::new(db.get_postgres_connection_pool().to_owned());
            store.migrate().await?;

            SessionBackend::Postgres(store)
        }
        #[cfg(feature = "sqlite")]
        DatabaseBackend::Sqlite => {
            let store = SqliteStore::new(db.get_sqlite_connection_pool().to_owned());
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

    let bookmark_repository = Arc::new(BookmarkSqlRepository::new(db.clone()));
    let collection_repository = Arc::new(CollectionSqlRepository::new(db.clone()));
    let feed_repository = Arc::new(FeedSqlRepository::new(db.clone()));
    let profile_repository = Arc::new(ProfileSqlRepository::new(db.clone()));
    let tag_repository = Arc::new(TagSqlRepository::new(db.clone()));

    let base64_decoder = Arc::new(Base64Encoder);

    let auth_service = Arc::new(AuthService::new(
        Arc::new(UserSqlRepository::new(db.clone())),
        profile_repository.clone(),
        Arc::new(ArgonHasher),
    ));
    let backup_service = Arc::new(BackupService::new(
        Arc::new(BackupSqlRepository::new(db.clone())),
        bookmark_repository.clone(),
        collection_repository.clone(),
        feed_repository.clone(),
        tag_repository.clone(),
        Arc::new(OpmlManager),
        Arc::new(NetscapeManager),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        bookmark_repository,
        Arc::new(register_bookmark_plugins()),
        base64_decoder.clone(),
    ));
    let cleanup_service = Arc::new(CleanupService::new(Arc::new(CleanupSqlRepository::new(
        db.clone(),
    ))));
    let collection_service = Arc::new(CollectionService::new(collection_repository));
    let feed_service = Arc::new(FeedService::new(
        feed_repository.clone(),
        feed_plugin_registry.clone(),
    ));
    let feed_entry_service = Arc::new(FeedEntryService::new(
        Arc::new(FeedEntrySqlRepository::new(db.clone())),
        base64_decoder,
    ));
    let profile_service = Arc::new(ProfileService::new(profile_repository.clone()));
    let refresh_service = Arc::new(RefreshService::new(
        feed_plugin_registry,
        feed_repository.clone(),
        profile_repository.clone(),
    ));
    let tag_service = Arc::new(TagService::new(tag_repository));

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(backup_service),
        BookmarkState::new(bookmark_service),
        CollectionState::new(collection_service),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        ProfileState::new(profile_service),
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
