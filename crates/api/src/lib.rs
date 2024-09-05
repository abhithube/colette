use auth::AuthState;
use axum::{extract::FromRef, routing, Router};
use backup::BackupState;
use bookmark::BookmarkState;
use colette_config::AppConfig;
use collection::CollectionState;
pub use common::Session;
use feed::FeedState;
use feed_entry::FeedEntryState;
use folder::FolderState;
use http::{header, HeaderValue, Method};
use profile::ProfileState;
use tag::TagState;
use tower_http::cors::CorsLayer;
use tower_sessions::{cookie::time::Duration, Expiry, SessionManagerLayer, SessionStore};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::AuthApi, backup::BackupApi, bookmark::BookmarkApi, collection::CollectionApi,
    common::BaseError, feed::FeedApi, feed_entry::FeedEntryApi, folder::FolderApi,
    profile::ProfileApi, tag::TagApi,
};

pub mod auth;
pub mod backup;
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
    backup_state: BackupState,
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
        backup_state: BackupState,
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
            backup_state,
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
      (url = "http://localhost:8000"),
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
        let (mut api, openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest(
                "/api/v1",
                OpenApiRouter::new()
                    .nest("/auth", AuthApi::router())
                    .with_state(AuthState::from_ref(self.api_state))
                    .nest("/backups", BackupApi::router())
                    .with_state(BackupState::from_ref(self.api_state))
                    .nest("/bookmarks", BookmarkApi::router())
                    .with_state(BookmarkState::from_ref(self.api_state))
                    .nest("/collections", CollectionApi::router())
                    .with_state(CollectionState::from_ref(self.api_state))
                    .nest("/feedEntries", FeedEntryApi::router())
                    .with_state(FeedEntryState::from_ref(self.api_state))
                    .nest("/feeds", FeedApi::router())
                    .with_state(FeedState::from_ref(self.api_state))
                    .nest("/folders", FolderApi::router())
                    .with_state(FolderState::from_ref(self.api_state))
                    .nest("/profiles", ProfileApi::router())
                    .with_state(ProfileState::from_ref(self.api_state))
                    .nest("/tags", TagApi::router())
                    .with_state(TagState::from_ref(self.api_state)),
            )
            .layer(
                SessionManagerLayer::new(self.session_store)
                    .with_secure(false)
                    .with_expiry(Expiry::OnInactivity(Duration::days(1))),
            )
            .split_for_parts();

        api = api
            .merge(Scalar::with_url("/api/v1/doc", openapi.clone()))
            .route(
                "/api/v1/openapi.json",
                routing::get(|| async move { openapi.to_pretty_json().unwrap() }),
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
