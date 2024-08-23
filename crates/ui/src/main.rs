#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;

    use colette_api::{
        auth::AuthState, bookmarks::BookmarksState, entries::EntriesState, feeds::FeedsState,
        profiles::ProfilesState, tags::TagsState, Api, ApiState,
    };
    use colette_backup::OpmlManager;
    use colette_migrations::{Migrator, MigratorTrait};
    use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
    use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
    use colette_session::{PostgresStore, SessionBackend, SqliteStore};
    use colette_sql::SqlRepository;
    use colette_tasks::handle_refresh_task;
    use colette_ui::{app::*, fileserv::file_and_error_handler};
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseBackend};
    use tower_sessions::ExpiredDeletion;

    const CRON_CLEANUP: &str = "0 0 0 * * *";

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

    let conf = get_configuration(None).await?;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    let auth_state = AuthState::new(
        Arc::new(UserSqlRepository::new(db.clone())),
        profile_repository.clone(),
    );
    let bookmark_state = BookmarkState::new(
        Arc::new(BookmarkSqlRepository::new(db.clone())),
        Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
    );
    let collection_state = CollectionState::new(Arc::new(CollectionSqlRepository::new(db.clone())));
    let feed_state = FeedState::new(feed_repository, feed_scraper, Arc::new(OpmlManager));
    let feed_entry_state = FeedEntryState::new(Arc::new(FeedEntrySqlRepository::new(db.clone())));
    let folder_state = FolderState::new(Arc::new(FolderSqlRepository::new(db.clone())));
    let profile_state = ProfileState::new(profile_repository);
    let tag_state = TagState::new(Arc::new(TagSqlRepository::new(db)));

    let api_state = ApiState::new(
        auth_state,
        bookmark_state,
        collection_state,
        feed_state,
        feed_entry_state,
        folder_state,
        profile_state,
        tag_state,
    );
    let app = Api::new(&api_state, &app_config, session_backend)
        .build()
        .with_state(api_state)
        .leptos_routes(&leptos_options, routes, App)
        .fallback(file_and_error_handler)
        .with_state(leptos_options.clone());

    let listener = tokio::net::TcpListener::bind(&leptos_options.site_addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    deletion_task.await??;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
