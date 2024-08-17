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
use colette_sql::SqlRepository;
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

    let repository = Arc::new(SqlRepository::new(db.clone()));

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

    if app_config.refresh_enabled {
        handle_refresh_task(
            &app_config.cron_refresh,
            feed_scraper.clone(),
            repository.clone(),
            repository.clone(),
        )
    }

    colette_tasks::handle_cleanup_task(CRON_CLEANUP, repository.clone());

    let api_state = ApiState {
        auth_state: AuthState {
            user_repository: repository.clone(),
            profile_repository: repository.clone(),
        },
        bookmark_state: BookmarkState {
            repository: repository.clone(),
            scraper: Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
        },
        collection_state: CollectionState {
            repository: repository.clone(),
        },
        feed_state: FeedState {
            repository: repository.clone(),
            scraper: feed_scraper,
            opml: Arc::new(OpmlManager),
        },
        feed_entry_state: FeedEntryState {
            repository: repository.clone(),
        },
        folder_state: FolderState {
            repository: repository.clone(),
        },
        profile_state: ProfileState {
            repository: repository.clone(),
        },
        tag_state: TagState { repository },
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
