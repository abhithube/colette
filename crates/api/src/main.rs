use std::{env, error::Error, sync::Arc};

use axum::Router;
use colette_core::{auth::AuthService, profiles::ProfilesService};
use colette_password::Argon2Hasher;
use colette_postgres::{ProfilesPostgresRepository, UsersPostgresRepository};
// use colette_sqlite::{ProfilesSqliteRepository, UsersSqliteRepository};
use tokio::{net::TcpListener, task};
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::PostgresStore;
// use tower_sessions_sqlx_store::SqliteStore;

mod api;
mod auth;
mod error;
mod profiles;
mod session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = env::var("DATABASE_URL")?;

    let pool = colette_postgres::create_database(&database_url).await?;
    // let pool = colette_sqlite::create_database(&database_url).await?;

    let session_store = PostgresStore::new(pool.clone());
    // let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await?;

    let deletion_task = task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    let users_repository = Arc::new(UsersPostgresRepository::new(pool.clone()));
    let profiles_repository = Arc::new(ProfilesPostgresRepository::new(pool.clone()));

    // let users_repository = Arc::new(UsersSqliteRepository::new(pool.clone()));
    // let profiles_repository = Arc::new(ProfilesSqliteRepository::new(pool.clone()));

    let argon_hasher = Arc::new(Argon2Hasher::default());
    let auth_service = Arc::new(AuthService::new(
        users_repository,
        profiles_repository.clone(),
        argon_hasher,
    ));
    let profiles_service = Arc::new(ProfilesService::new(profiles_repository));

    let state = api::Context {
        auth_service,
        profiles_service,
    };

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(auth::router())
                .merge(profiles::router())
                .with_state(state),
        )
        .layer(session_layer);

    let listener = TcpListener::bind("localhost:3001").await?;
    axum::serve(listener, app).await?;

    deletion_task.await??;

    Ok(())
}
