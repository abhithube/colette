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
    feed::FeedState, feed_entry::FeedEntryState, folder::FolderState, profile::ProfileState,
    tag::TagState, Api, ApiState,
};
use colette_backup::opml::OpmlManager;
use colette_core::{
    auth::AuthService, backup::BackupService, bookmark::BookmarkService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    folder::FolderService, profile::ProfileService, refresh::RefreshService, tag::TagService,
};
use colette_migration::{Migrator, MigratorTrait};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_repository::{
    BackupSqlRepository, BookmarkSqlRepository, CollectionSqlRepository, FeedEntrySqlRepository,
    FeedSqlRepository, FolderSqlRepository, ProfileSqlRepository, TagSqlRepository,
    UserSqlRepository,
};
use colette_scraper::{bookmark::DefaultBookmarkScraper, feed::DefaultFeedScraper};
#[cfg(feature = "postgres")]
use colette_session::PostgresStore;
use colette_session::SessionBackend;
#[cfg(feature = "sqlite")]
use colette_session::SqliteStore;
use colette_utils::password::ArgonHasher;
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

    let feed_scraper = Arc::new(DefaultFeedScraper::new(register_feed_plugins()));

    let feed_repository = Arc::new(FeedSqlRepository::new(db.clone()));
    let profile_repository = Arc::new(ProfileSqlRepository::new(db.clone()));

    if app_config.refresh_enabled {
        let schedule = Schedule::from_str(&app_config.cron_refresh)?;

        let worker = WorkerBuilder::new("refresh-feeds")
            .data(Arc::new(RefreshService::new(
                feed_scraper.clone(),
                feed_repository.clone(),
                profile_repository.clone(),
            )))
            .backend(CronStream::new(schedule))
            .build_fn(colette_tasks::refresh_feeds);

        Monitor::<TokioExecutor>::new()
            .register(worker)
            .run()
            .await?;
    }

    colette_tasks::handle_cleanup_task(CRON_CLEANUP, feed_repository.clone());

    let auth_service = Arc::new(AuthService::new(
        Arc::new(UserSqlRepository::new(db.clone())),
        profile_repository.clone(),
        Arc::new(ArgonHasher),
    ));
    let backup_service = Arc::new(BackupService::new(
        Arc::new(BackupSqlRepository::new(db.clone())),
        feed_repository.clone(),
        Arc::new(OpmlManager),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        Arc::new(BookmarkSqlRepository::new(db.clone())),
        Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
    ));
    let collection_service = Arc::new(CollectionService::new(Arc::new(
        CollectionSqlRepository::new(db.clone()),
    )));
    let feed_service = Arc::new(FeedService::new(feed_repository, feed_scraper));
    let feed_entry_service = Arc::new(FeedEntryService::new(Arc::new(
        FeedEntrySqlRepository::new(db.clone()),
    )));
    let folder_service = Arc::new(FolderService::new(Arc::new(FolderSqlRepository::new(
        db.clone(),
    ))));
    let profile_service = Arc::new(ProfileService::new(profile_repository));
    let tag_service = Arc::new(TagService::new(Arc::new(TagSqlRepository::new(db))));

    let api_state = ApiState::new(
        AuthState::new(auth_service),
        BackupState::new(backup_service),
        BookmarkState::new(bookmark_service),
        CollectionState::new(collection_service),
        FeedState::new(feed_service),
        FeedEntryState::new(feed_entry_service),
        FolderState::new(folder_service),
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
    axum::serve(listener, api).await?;

    deletion_task.await??;

    Ok(())
}
