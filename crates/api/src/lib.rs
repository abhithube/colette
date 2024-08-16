use auth::AuthState;
use axum::{
    extract::FromRef,
    http::{header, HeaderValue, Method},
    routing, Router,
};
use bookmarks::BookmarksState;
use colette_config::AppConfig;
pub use common::Session;
use entries::EntriesState;
use feeds::FeedsState;
use profiles::ProfilesState;
use tags::TagsState;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer, SessionStore};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::Api as Auth, bookmarks::Api as Bookmarks, common::BaseError, entries::Api as Entries,
    feeds::Api as Feeds, profiles::Api as Profiles, tags::Api as Tags,
};

pub mod auth;
pub mod bookmarks;
mod common;
pub mod entries;
pub mod feeds;
pub mod profiles;
pub mod tags;

#[derive(Clone, FromRef)]
pub struct ApiState {
    pub auth_state: AuthState,
    pub bookmarks_state: BookmarksState,
    pub entries_state: EntriesState,
    pub feeds_state: FeedsState,
    pub profiles_state: ProfilesState,
    pub tags_state: TagsState,
}

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
  components(schemas(BaseError))
)]
struct ApiDoc;

pub struct Api<'a, Store: SessionStore + Clone> {
    api_state: &'a ApiState,
    app_config: &'a AppConfig,
    session_store: Store,
}

impl<'a, Store: SessionStore + Clone> Api<'a, Store> {
    pub fn new(api_state: &'a ApiState, app_config: &'a AppConfig, session_store: Store) -> Self {
        Self {
            api_state,
            app_config,
            session_store,
        }
    }

    pub fn build(self) -> Router<ApiState> {
        let mut api = Router::new()
            .nest(
                "/api/v1",
                Router::new()
                    .merge(Scalar::with_url("/doc", ApiDoc::openapi()))
                    .route(
                        "/openapi.json",
                        routing::get(|| async { ApiDoc::openapi().to_pretty_json().unwrap() }),
                    )
                    .merge(Auth::router())
                    .with_state(AuthState::from_ref(self.api_state))
                    .merge(Bookmarks::router())
                    .with_state(BookmarksState::from_ref(self.api_state))
                    .merge(Entries::router())
                    .with_state(EntriesState::from_ref(self.api_state))
                    .merge(Feeds::router())
                    .with_state(FeedsState::from_ref(self.api_state))
                    .merge(Profiles::router())
                    .with_state(ProfilesState::from_ref(self.api_state))
                    .merge(Tags::router())
                    .with_state(TagsState::from_ref(self.api_state)),
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

            api = api.layer(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
                    .allow_origin(origins)
                    .allow_headers([header::CONTENT_TYPE])
                    .allow_credentials(true),
            )
        }

        api
    }
}
