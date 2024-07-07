use std::{collections::HashMap, env, error::Error, sync::Arc};

use axum::{routing, Router};
use colette_core::{auth::AuthService, feeds::FeedsService, profiles::ProfilesService};
use colette_password::Argon2Hasher;
use colette_postgres::{
    FeedsPostgresRepository, ProfilesPostgresRepository, UsersPostgresRepository,
};
use colette_scraper::{
    AtomExtractorOptions, DefaultDownloader, DefaultFeedExtractor, DefaultFeedPostprocessor,
    ExtractorOptions, FeedScraper, PluginRegistry,
};
use common::{FeedList, ProfileList};
// use colette_sqlite::{FeedsSqliteRepository, ProfilesSqliteRepository, UsersSqliteRepository};
use tokio::{net::TcpListener, task};
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, SessionManagerLayer,
};
// use tower_sessions_sqlx_store::SqliteStore;
use tower_sessions_sqlx_store::PostgresStore;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

mod auth;
mod common;
mod error;
mod feeds;
mod profiles;
mod session;
mod validation;

#[derive(OpenApi)]
#[openapi(
    servers(
        (url = "http://localhost:3001")
    ),
    nest(
        (path = "/api/v1/auth", api = auth::Api),
        (path = "/api/v1/feeds", api = feeds::Api),
        (path = "/api/v1/profiles", api = profiles::Api)
    ),
    components(schemas(common::Error, FeedList, ProfileList)),
    tags(
        (name = "Auth"),
        (name = "Feeds"),
        (name = "Profiles")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let database_url = env::var("DATABASE_URL")?;

    let pool = colette_postgres::create_database(&database_url).await?;
    // let pool = colette_sqlite::create_database(&database_url).await?;

    let users_repository = Box::new(UsersPostgresRepository::new(pool.clone()));
    let profiles_repository = Box::new(ProfilesPostgresRepository::new(pool.clone()));
    let feeds_repository = Box::new(FeedsPostgresRepository::new(pool.clone()));
    // let users_repository = Box::new(UsersSqliteRepository::new(pool.clone()));
    // let profiles_repository = Box::new(ProfilesSqliteRepository::new(pool.clone()));
    // let feeds_repository = Box::new(FeedsSqliteRepository::new(pool.clone()));

    let argon_hasher = Box::new(Argon2Hasher::default());

    let auth_service = Arc::new(AuthService::new(
        users_repository,
        profiles_repository.clone(),
        argon_hasher,
    ));
    let profiles_service = Arc::new(ProfilesService::new(profiles_repository));
    let feeds_service = Arc::new(FeedsService::new(feeds_repository, feed_scraper));

    let session_store = PostgresStore::new(pool.clone());
    // let session_store = SqliteStore::new(pool.clone());

    session_store.migrate().await?;

    let deletion_task = task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let state = common::Context {
        auth_service,
        profiles_service,
        feeds_service,
    };

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .merge(Scalar::with_url("/doc", ApiDoc::openapi()))
                .route("/openapi.json", routing::get(doc))
                .merge(auth::Api::router())
                .merge(feeds::Api::router())
                .merge(profiles::Api::router())
                .with_state(state),
        )
        .layer(
            SessionManagerLayer::new(session_store)
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        );

    let listener = TcpListener::bind("localhost:3001").await?;
    axum::serve(listener, app).await?;

    deletion_task.await??;

    Ok(())
}

async fn doc() -> Result<String, error::Error> {
    ApiDoc::openapi()
        .to_pretty_json()
        .map_err(|_| error::Error::Unknown)
}
