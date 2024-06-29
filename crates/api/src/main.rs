use std::{env, error::Error, sync::Arc};

use axum::Router;
use colette_core::auth::AuthService;
use colette_password::Argon2Hasher;
use colette_postgres::repositories::users::UsersPostgresRepository;
use tokio::net::TcpListener;

mod api;
mod auth;
mod error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let database_url = env::var("DATABASE_URL")?;

    let pool = colette_postgres::create_database(&database_url).await?;

    let users_repository = Arc::new(UsersPostgresRepository::new(pool.clone()));

    let argon_hasher = Arc::new(Argon2Hasher::default());
    let auth_service = Arc::new(AuthService::new(users_repository, argon_hasher));

    let state = api::Context { auth_service };

    let app = Router::new().nest(
        "/api",
        Router::new().merge(auth::router()).with_state(state),
    );

    let listener = TcpListener::bind("localhost:3001").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
