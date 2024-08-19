use std::{error::Error, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, bookmark::BookmarkState, collection::CollectionState, feed::FeedState,
    feed_entry::FeedEntryState, folder::FolderState, profile::ProfileState, tag::TagState, Api,
    ApiState,
};
use colette_backup::opml::OpmlManager;
use colette_migrations::{Migrator, MigratorTrait};
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
use colette_session::{PostgresStore, SessionBackend, SqliteStore};
use colette_sql::{
    BookmarkSqlRepository, CollectionSqlRepository, FeedEntrySqlRepository, FeedSqlRepository,
    FolderSqlRepository, ProfileSqlRepository, TagSqlRepository, UserSqlRepository,
};
use colette_tasks::handle_refresh_task;
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

    let api_state = ApiState {
        auth_state: AuthState {
            user_repository: Arc::new(UserSqlRepository::new(db.clone())),
            profile_repository: profile_repository.clone(),
        },
        bookmark_state: BookmarkState {
            repository: Arc::new(BookmarkSqlRepository::new(db.clone())),
            scraper: Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
        },
        collection_state: CollectionState {
            repository: Arc::new(CollectionSqlRepository::new(db.clone())),
        },
        feed_state: FeedState {
            repository: feed_repository,
            scraper: feed_scraper,
            opml: Arc::new(OpmlManager),
        },
        feed_entry_state: FeedEntryState {
            repository: Arc::new(FeedEntrySqlRepository::new(db.clone())),
        },
        folder_state: FolderState {
            repository: Arc::new(FolderSqlRepository::new(db.clone())),
        },
        profile_state: ProfileState {
            repository: profile_repository,
        },
        tag_state: TagState {
            repository: Arc::new(TagSqlRepository::new(db)),
        },
    };

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
