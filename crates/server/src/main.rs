use std::{error::Error, str::FromStr, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use chrono::Local;
use colette_api::{App, AppState};
use colette_backup::OpmlManager;
use colette_core::{
    auth::AuthService, bookmarks::BookmarksService, entries::EntriesService, feeds::FeedsService,
    profiles::ProfilesService, tags::TagsService, utils::task::Task,
};
use colette_password::Argon2Hasher;
use colette_plugins::{register_bookmark_plugins, register_feed_plugins};
use colette_postgres::PostgresRepository;
use colette_scraper::{DefaultBookmarkScraper, DefaultFeedScraper};
use colette_tasks::{CleanupTask, RefreshTask};
use cron::Schedule;
use tokio::{net::TcpListener, time};
use tower_sessions::ExpiredDeletion;
use tower_sessions_sqlx_store::PostgresStore;

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../packages/web/dist"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let app_config = colette_config::load_config()?;

    let pool = colette_postgres::initialize(&app_config.database_url).await?;

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
        let feed_scraper = feed_scraper.clone();
        let repository = repository.clone();

        let schedule = Schedule::from_str(&app_config.cron_refresh).unwrap();

        tokio::spawn(async move {
            let refresh_task =
                RefreshTask::new(feed_scraper.clone(), repository.clone(), repository.clone());

            loop {
                let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
                let duration = (upcoming - Local::now()).to_std().unwrap();

                time::sleep(duration).await;

                let start = Local::now();
                println!("Started refresh task at: {}", start);

                match refresh_task.run().await {
                    Ok(_) => {
                        let elasped = (Local::now().time() - start.time()).num_milliseconds();
                        println!("Finished refresh task in {} ms", elasped);
                    }
                    Err(e) => {
                        println!("Failed refresh task: {}", e);
                    }
                }
            }
        });
    }

    {
        let repository = repository.clone();

        let schedule = Schedule::from_str(CRON_CLEANUP).unwrap();

        tokio::spawn(async move {
            let cleanup_task = CleanupTask::new(repository.clone());

            loop {
                let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
                let duration = (upcoming - Local::now()).to_std().unwrap();

                time::sleep(duration).await;

                let start = Local::now();
                println!("Started cleanup task at: {}", start);

                match cleanup_task.run().await {
                    Ok(_) => {
                        let elasped = (Local::now().time() - start.time()).num_milliseconds();
                        println!("Finished cleanup task in {} ms", elasped);
                    }
                    Err(e) => {
                        println!("Failed cleanup task: {}", e);
                    }
                }
            }
        });
    }

    let app_state = AppState {
        auth_service: AuthService::new(
            repository.clone(),
            repository.clone(),
            Arc::new(Argon2Hasher {}),
        )
        .into(),
        bookmark_service: BookmarksService::new(
            repository.clone(),
            Arc::new(DefaultBookmarkScraper::new(register_bookmark_plugins())),
        )
        .into(),
        entries_service: EntriesService::new(repository.clone()).into(),
        feeds_service: FeedsService::new(repository.clone(), feed_scraper, Arc::new(OpmlManager))
            .into(),
        profiles_service: ProfilesService::new(repository.clone()).into(),
        tags_service: TagsService::new(repository).into(),
    };

    let api = App::new(app_state, &app_config, session_store)
        .build()
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
