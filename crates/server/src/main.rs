use std::{error::Error, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, bookmark::BookmarkState, collection::CollectionState, feed::FeedState,
    feed_entry::FeedEntryState, folder::FolderState, profile::ProfileState, tag::TagState, Api,
    ApiState,
};
use colette_backup::opml::OpmlManager;
use colette_core::{
    auth::AuthService, bookmark::BookmarkService, collection::CollectionService, feed::FeedService,
    feed_entry::FeedEntryService, folder::FolderService, profile::ProfileService, tag::TagService,
};
use colette_migrations::{Migrator, MigratorTrait};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_repositories::{
    BookmarkSqlRepository, CollectionSqlRepository, FeedEntrySqlRepository, FeedSqlRepository,
    FolderSqlRepository, ProfileSqlRepository, TagSqlRepository, UserSqlRepository,
};
use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
use colette_session::{PostgresStore, SessionBackend, SqliteStore};
use colette_tasks::handle_refresh_task;
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
        DatabaseBackend::Postgres => {
            let store = PostgresStore::new(db.get_postgres_connection_pool().to_owned());
            store.migrate().await?;

            SessionBackend::Postgres(store)
        }
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
        handle_refresh_task(
            &app_config.cron_refresh,
            feed_scraper.clone(),
            feed_repository.clone(),
            profile_repository.clone(),
        )
    }

    colette_tasks::handle_cleanup_task(CRON_CLEANUP, feed_repository.clone());

    let auth_service = Arc::new(AuthService::new(
        Arc::new(UserSqlRepository::new(db.clone())),
        profile_repository.clone(),
        Arc::new(ArgonHasher),
    ));
    let bookmark_service = Arc::new(BookmarkService::new(
        Arc::new(BookmarkSqlRepository::new(db.clone())),
        Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
    ));
    let collection_service = Arc::new(CollectionService::new(Arc::new(
        CollectionSqlRepository::new(db.clone()),
    )));
    let feed_service = Arc::new(FeedService::new(
        feed_repository,
        feed_scraper,
        Arc::new(OpmlManager),
    ));
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
