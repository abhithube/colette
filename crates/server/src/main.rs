use std::{collections::HashMap, env, error::Error, str::FromStr, sync::Arc};

use axum::{
    http::{header, HeaderValue, Method},
    routing, Router,
};
use colette_core::{
    auth::AuthService, entries::EntriesService, feeds::FeedsService, profiles::ProfilesService,
};
use colette_password::Argon2Hasher;
#[cfg(feature = "postgres")]
use colette_postgres::{
    iterate_feeds, EntriesPostgresRepository, FeedsPostgresRepository, ProfilesPostgresRepository,
    UsersPostgresRepository,
};
use colette_scraper::{
    AtomExtractorOptions, DefaultDownloader, DefaultFeedExtractor, DefaultFeedPostprocessor,
    ExtractorOptions, FeedScraper, PluginRegistry,
};
#[cfg(feature = "sqlite")]
use colette_sqlite::{
    iterate_feeds, EntriesSqliteRepository, FeedsSqliteRepository, ProfilesSqliteRepository,
    UsersSqliteRepository,
};
use common::{EntryList, FeedList, ProfileList};
use cron::Schedule;
use futures::stream::StreamExt;
use tokio::{net::TcpListener, task};
use tokio_cron_scheduler::{Job, JobScheduler};
use tower_http::cors::CorsLayer;
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, SessionManagerLayer,
};
#[cfg(feature = "postgres")]
use tower_sessions_sqlx_store::PostgresStore;
#[cfg(feature = "sqlite")]
use tower_sessions_sqlx_store::SqliteStore;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};
use web::static_handler;

mod auth;
mod common;
mod entries;
mod error;
mod feeds;
mod profiles;
mod session;
mod validation;
mod web;

const DEFAULT_PORT: u32 = 8000;
const DEFAULT_CRON_REFRESH: &str = "0 */15 * * * * *";

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:8000")
    ),
    nest(
        (path = "/api/v1/auth", api = auth::Api),
        (path = "/api/v1/entries", api = entries::Api),
        (path = "/api/v1/feeds", api = feeds::Api),
        (path = "/api/v1/profiles", api = profiles::Api)
    ),
    components(schemas(common::Error, EntryList, FeedList, ProfileList)),
    tags(
        (name = "Auth"),
        (name = "Entries"),
        (name = "Feeds"),
        (name = "Profiles")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port = env::var("PORT")
        .map(|e| e.parse::<u32>())
        .unwrap_or(Ok(DEFAULT_PORT))?;
    let database_url = env::var("DATABASE_URL")?;
    let origin_urls = env::var("ORIGIN_URLS").ok();
    let cron_refresh = env::var("CRON_REFRESH").unwrap_or(String::from(DEFAULT_CRON_REFRESH));

    #[cfg(feature = "postgres")]
    let pool = colette_postgres::create_database(&database_url).await?;
    #[cfg(feature = "sqlite")]
    let pool = colette_sqlite::create_database(&database_url).await?;

    #[cfg(feature = "postgres")]
    let session_store = PostgresStore::new(pool.clone());
    #[cfg(feature = "sqlite")]
    let session_store = SqliteStore::new(pool.clone());

    #[cfg(feature = "postgres")]
    let (users_repository, profiles_repository, feeds_repository, entries_repository) = (
        Box::new(UsersPostgresRepository::new(pool.clone())),
        Box::new(ProfilesPostgresRepository::new(pool.clone())),
        Box::new(FeedsPostgresRepository::new(pool.clone())),
        Box::new(EntriesPostgresRepository::new(pool.clone())),
    );
    #[cfg(feature = "sqlite")]
    let (users_repository, profiles_repository, feeds_repository, entries_repository) = (
        Box::new(UsersSqliteRepository::new(pool.clone())),
        Box::new(ProfilesSqliteRepository::new(pool.clone())),
        Box::new(FeedsSqliteRepository::new(pool.clone())),
        Box::new(EntriesSqliteRepository::new(pool.clone())),
    );

    let schedule = Schedule::from_str(&cron_refresh)?;
    let scheduler = JobScheduler::new().await?;

    scheduler
        .add(Job::new_async(schedule, move |_id, _scheduler| {
            let pool = pool.clone();

            Box::pin(async move {
                let mut stream = iterate_feeds(&pool);
                while let Some(row) = stream.next().await {
                    let url = row.unwrap();
                    println!("{}", url);
                }
            })
        })?)
        .await?;

    scheduler.start().await?;

    let downloader = Box::new(DefaultDownloader {});
    let feed_extractor = Box::new(DefaultFeedExtractor {
        options: ExtractorOptions {
            ..AtomExtractorOptions::default().inner()
        },
    });
    let feed_postprocessor = Box::new(DefaultFeedPostprocessor {});

    let feed_registry = PluginRegistry {
        downloaders: HashMap::new(),
        extractors: HashMap::new(),
        postprocessors: HashMap::new(),
    };
    let feed_scraper = Box::new(FeedScraper::new(
        feed_registry,
        downloader,
        feed_extractor,
        feed_postprocessor,
    ));

    let argon_hasher = Box::new(Argon2Hasher::default());

    let auth_service = Arc::new(AuthService::new(
        users_repository,
        profiles_repository.clone(),
        argon_hasher,
    ));
    let entries_service = Arc::new(EntriesService::new(entries_repository));
    let feeds_service = Arc::new(FeedsService::new(feeds_repository, feed_scraper));
    let profiles_service = Arc::new(ProfilesService::new(profiles_repository));

    session_store.migrate().await?;

    let deletion_task = task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let state = common::Context {
        auth_service,
        entries_service,
        feeds_service,
        profiles_service,
    };

    let mut app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .merge(Scalar::with_url("/doc", ApiDoc::openapi()))
                .route(
                    "/openapi.json",
                    routing::get(|| async { ApiDoc::openapi().to_pretty_json().unwrap() }),
                )
                .merge(auth::Api::router())
                .merge(entries::Api::router())
                .merge(feeds::Api::router())
                .merge(profiles::Api::router())
                .with_state(state),
        )
        .fallback_service(routing::get(static_handler))
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        );

    if let Some(origin_urls) = origin_urls {
        let mut origins: Vec<HeaderValue> = vec![];
        for part in origin_urls.split(",") {
            let origin = part.parse::<HeaderValue>()?;
            origins.push(origin);
        }

        app = app.layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                .allow_origin(origins)
                .allow_headers([header::CONTENT_TYPE])
                .allow_credentials(true),
        )
    }

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;

    deletion_task.await??;

    Ok(())
}
