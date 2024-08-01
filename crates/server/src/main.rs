use std::{error::Error, str::FromStr, sync::Arc};

use axum_embed::{FallbackBehavior, ServeEmbed};
use chrono::Local;
use colette_api::{App, Context};
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
use sqlx::PgPool;
use tokio::{net::TcpListener, time};
use tower_sessions::ExpiredDeletion;
use tower_sessions_sqlx_store::PostgresStore;

const CRON_CLEANUP: &str = "0 0 0 * * *";

#[derive(Clone, rust_embed::Embed)]
#[folder = "$CARGO_MANIFEST_DIR/../../packages/web/dist"]
struct Asset;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = colette_config::load_config()?;

    let pool = PgPool::connect(&config.database_url).await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let repository = Arc::new(PostgresRepository::new(pool.clone()));

    let store = PostgresStore::new(pool.clone());
    store.migrate().await?;

    let deletion_task = tokio::task::spawn(
        store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let feed_scraper = Arc::new(DefaultFeedScraper::new(register_feed_plugins()));

    if config.refresh_enabled {
        let feed_scraper = feed_scraper.clone();
        let repository = repository.clone();

        let schedule = Schedule::from_str(&config.cron_refresh).unwrap();

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

    let state = Context {
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

    let app = App::new(state, &config, store)
        .build_router()
        .fallback_service(ServeEmbed::<Asset>::with_parameters(
            Some(String::from("index.html")),
            FallbackBehavior::Ok,
            None,
        ));

    let listener = TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
    axum::serve(listener, app).await?;

    deletion_task.await??;

    Ok(())
}
