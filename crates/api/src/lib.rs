use axum::{
    http::{header, HeaderValue, Method},
    routing, Router,
};
use colette_config::AppConfig;
pub use common::{AppState, Session};
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer, SessionStore};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::Api as Auth,
    bookmarks::Api as Bookmarks,
    common::{BaseError, BookmarkList, EntryList, FeedDetectedList, FeedList, ProfileList},
    entries::Api as Entries,
    feeds::Api as Feeds,
    profiles::Api as Profiles,
    tags::Api as Tags,
};

pub mod auth;
pub mod bookmarks;
mod common;
pub mod entries;
pub mod feeds;
pub mod profiles;
pub mod tags;

#[derive(utoipa::OpenApi)]
#[openapi(
  servers(
      (url = "http://localhost:8000/api/v1")
  ),
  nest(
      (path = "/auth", api = Auth),
      (path = "/bookmarks", api = Bookmarks),
      (path = "/entries", api = Entries),
      (path = "/feeds", api = Feeds),
      (path = "/profiles", api = Profiles),
      (path = "/tags", api = Tags)
  ),
  components(schemas(BaseError, BookmarkList, FeedDetectedList, EntryList, FeedList, ProfileList))
)]
struct ApiDoc;

pub struct App<'a, Store: SessionStore + Clone> {
    app_state: AppState,
    app_config: &'a AppConfig,
    session_store: Store,
}

impl<'a, Store: SessionStore + Clone> App<'a, Store> {
    pub fn new(app_state: AppState, app_config: &'a AppConfig, session_store: Store) -> Self {
        Self {
            app_state,
            app_config,
            session_store,
        }
    }

    pub fn build(self) -> Router {
        let mut app = Router::new()
            .nest(
                "/api/v1",
                Router::new()
                    .merge(Scalar::with_url("/doc", ApiDoc::openapi()))
                    .route(
                        "/openapi.json",
                        routing::get(|| async { ApiDoc::openapi().to_pretty_json().unwrap() }),
                    )
                    .merge(Auth::router())
                    .merge(Bookmarks::router())
                    .merge(Entries::router())
                    .merge(Feeds::router())
                    .merge(Profiles::router())
                    .merge(Tags::router())
                    .with_state(self.app_state),
            )
            .layer(
                SessionManagerLayer::new(self.session_store)
                    .with_secure(false)
                    .with_expiry(Expiry::OnInactivity(Duration::days(1))),
            );

        if !self.app_config.origin_urls.is_empty() {
            let origins = self
                .app_config
                .origin_urls
                .iter()
                .filter_map(|e| e.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>();

            app = app.layer(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                    .allow_origin(origins)
                    .allow_headers([header::CONTENT_TYPE])
                    .allow_credentials(true),
            )
        }

        app
    }
}
