use std::sync::Arc;

use axum::{body::Body, http::Response, Router};
use colette_api::auth::{AuthApi, AuthState};
use colette_core::auth::AuthService;
use colette_util::password::ArgonHasher;
use d1::{profile::D1ProfileRepository, user::D1UserRepository};
use kv::store::KvSessionStore;
use tower::Service;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer};
use worker::{Context, Env, HttpRequest};

mod d1;
mod kv;

#[worker::event(fetch)]
async fn fetch(req: HttpRequest, env: Env, _ctx: Context) -> worker::Result<Response<Body>> {
    console_error_panic_hook::set_once();

    let d1 = Arc::new(env.d1("DB")?);
    let kv = env.kv("KV")?;

    let user_repository = Box::new(D1UserRepository::new(d1.clone()));
    let profile_repository = Box::new(D1ProfileRepository::new(d1.clone()));

    let auth_state = AuthState::new(AuthService::new(
        user_repository,
        profile_repository,
        Box::new(ArgonHasher),
    ));

    let mut router = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .nest("/auth", AuthApi::router().into())
                .with_state(auth_state),
        )
        .layer(
            SessionManagerLayer::new(KvSessionStore::new(kv))
                .with_secure(false)
                .with_expiry(Expiry::OnInactivity(Duration::days(1))),
        );

    let resp = router.call(req).await?;

    Ok(resp)
}
