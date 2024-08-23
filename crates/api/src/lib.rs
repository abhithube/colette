use auth::AuthState;
use axum::{
    extract::FromRef,
    http::{header, HeaderValue, Method},
    routing, Router,
};
use bookmark::BookmarkState;
use colette_config::AppConfig;
use collection::CollectionState;
pub use common::Session;
use feed::FeedState;
use feed_entry::FeedEntryState;
use folder::FolderState;
use profile::ProfileState;
use tag::TagState;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer, SessionStore};
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::Api as Auth, bookmark::Api as Bookmarks, collection::Api as Collections,
    common::BaseError, feed::Api as Feeds, feed_entry::Api as FeedEntries, folder::Api as Folders,
    profile::Api as Profiles, tag::Api as Tags,
};

pub mod auth;
pub mod bookmark;
pub mod collection;
mod common;
pub mod feed;
pub mod feed_entry;
pub mod folder;
pub mod profile;
pub mod tag;

#[derive(Clone, FromRef)]
pub struct ApiState {
    auth_state: AuthState,
    bookmark_state: BookmarkState,
    collection_state: CollectionState,
    feed_state: FeedState,
    feed_entry_state: FeedEntryState,
    folder_state: FolderState,
    profile_state: ProfileState,
    tag_state: TagState,
}

impl ApiState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth_state: AuthState,
        bookmark_state: BookmarkState,
        collection_state: CollectionState,
        feed_state: FeedState,
        feed_entry_state: FeedEntryState,
        folder_state: FolderState,
        profile_state: ProfileState,
        tag_state: TagState,
    ) -> Self {
        Self {
            auth_state,
            bookmark_state,
            collection_state,
            feed_state,
            feed_entry_state,
            folder_state,
            profile_state,
            tag_state,
        }
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(
  servers(
      (url = "http://localhost:8000/api/v1")
  ),
  nest(
      (path = "/auth", api = Auth, tags = ["Auth"]),
      (path = "/bookmarks", api = Bookmarks, tags = ["Bookmarks"]),
      (path = "/collections", api = Collections, tags = ["Collections"]),
      (path = "/feeds", api = Feeds, tags = ["Feeds"]),
      (path = "/feedEntries", api = FeedEntries, tags = ["Feed Entries"]),
      (path = "/folders", api = Folders, tags = ["Folders"]),
      (path = "/profiles", api = Profiles, tags = ["Profiles"]),
      (path = "/tags", api = Tags, tags = ["Tags"])
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
                    .with_state(BookmarkState::from_ref(self.api_state))
                    .merge(Collections::router())
                    .with_state(CollectionState::from_ref(self.api_state))
                    .merge(Feeds::router())
                    .with_state(FeedState::from_ref(self.api_state))
                    .merge(FeedEntries::router())
                    .with_state(FeedEntryState::from_ref(self.api_state))
                    .merge(Folders::router())
                    .with_state(FolderState::from_ref(self.api_state))
                    .merge(Profiles::router())
                    .with_state(ProfileState::from_ref(self.api_state))
                    .merge(Tags::router())
                    .with_state(TagState::from_ref(self.api_state)),
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
