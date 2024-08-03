use std::{error::Error, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use colette_api::{
    auth::AuthState, bookmarks::BookmarksState, entries::EntriesState, feeds::FeedsState,
    profiles::ProfilesState, tags::TagsState, Api, ApiState,
};
use colette_backup::OpmlManager;
use colette_core::{
    auth::AuthService, bookmarks::BookmarksService, entries::EntriesService, feeds::FeedsService,
    profiles::ProfilesService, tags::TagsService,
};
use colette_db::PostgresRepository;
use colette_password::Argon2Hasher;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
use colette_tasks::handle_refresh_task;
use tokio::net::TcpListener;
use tower_sessions::ExpiredDeletion;
use tower_sessions_sqlx_store::PostgresStore;

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../packages/web/dist"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_config = colette_config::load_config()?;

    let pool = colette_db::initialize(&app_config.database_url).await?;

    let repository = Arc::new(PostgresRepository::new(pool.clone()));

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        session_store
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
            service: AuthService::new(
                repository.clone(),
                repository.clone(),
                Arc::new(Argon2Hasher {}),
            )
            .into(),
        },
        bookmarks_state: BookmarksState {
            service: BookmarksService::new(
                repository.clone(),
                Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
            )
            .into(),
        },
        entries_state: EntriesState {
            service: EntriesService::new(repository.clone()).into(),
        },
        feeds_state: FeedsState {
            service: FeedsService::new(repository.clone(), feed_scraper, Arc::new(OpmlManager))
                .into(),
        },
        profiles_state: ProfilesState {
            service: ProfilesService::new(repository.clone()).into(),
        },
        tags_state: TagsState {
            service: TagsService::new(repository).into(),
        },
    };

    let api = Api::new(&api_state, &app_config, session_store)
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
