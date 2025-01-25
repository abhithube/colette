use std::ops::RangeFull;

use auth::AuthState;
use axum::{extract::FromRef, routing, Router};
use backup::BackupState;
use bookmark::BookmarkState;
pub use common::{Paginated, Session};
use feed::FeedState;
use feed_entry::FeedEntryState;
use folder::{FolderApi, FolderState};
use library::{LibraryApi, LibraryState};
use tag::TagState;
use utoipa::{openapi::Server, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    auth::AuthApi, backup::BackupApi, bookmark::BookmarkApi, common::BaseError, feed::FeedApi,
    feed_entry::FeedEntryApi, tag::TagApi,
};

pub mod auth;
pub mod backup;
pub mod bookmark;
// pub mod collection;
mod common;
pub mod feed;
pub mod feed_entry;
pub mod folder;
pub mod library;
// pub mod smart_feed;
pub mod tag;

#[derive(Clone, FromRef)]
pub struct ApiState {
    auth_state: AuthState,
    backup_state: BackupState,
    bookmark_state: BookmarkState,
    // collection_state: CollectionState,
    feed_state: FeedState,
    feed_entry_state: FeedEntryState,
    folder_state: FolderState,
    library_state: LibraryState,
    // smart_feed_state: SmartFeedState,
    tag_state: TagState,
}

impl ApiState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        auth_state: AuthState,
        backup_state: BackupState,
        bookmark_state: BookmarkState,
        // collection_state: CollectionState,
        feed_state: FeedState,
        feed_entry_state: FeedEntryState,
        folder_state: FolderState,
        library_state: LibraryState,
        // smart_feed_state: SmartFeedState,
        tag_state: TagState,
    ) -> Self {
        Self {
            auth_state,
            backup_state,
            bookmark_state,
            // collection_state,
            feed_state,
            feed_entry_state,
            folder_state,
            library_state,
            // smart_feed_state,
            tag_state,
        }
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(components(schemas(BaseError)))]
struct ApiDoc;

pub struct Api<'a> {
    api_state: &'a ApiState,
    api_prefix: &'a str,
}

impl<'a> Api<'a> {
    pub fn new(api_state: &'a ApiState, api_prefix: &'a str) -> Self {
        Self {
            api_state,
            api_prefix,
        }
    }

    pub fn build(self) -> Router<ApiState> {
        let (api, mut openapi) = OpenApiRouter::with_openapi(ApiDoc::openapi())
            .nest(
                self.api_prefix,
                OpenApiRouter::new()
                    .nest("/auth", AuthApi::router())
                    .with_state(AuthState::from_ref(self.api_state))
                    .nest("/backups", BackupApi::router())
                    .with_state(BackupState::from_ref(self.api_state))
                    .nest("/bookmarks", BookmarkApi::router())
                    .with_state(BookmarkState::from_ref(self.api_state))
                    // .nest("/collections", CollectionApi::router())
                    // .with_state(CollectionState::from_ref(self.api_state))
                    .nest("/feedEntries", FeedEntryApi::router())
                    .with_state(FeedEntryState::from_ref(self.api_state))
                    .nest("/feeds", FeedApi::router())
                    .with_state(FeedState::from_ref(self.api_state))
                    .nest("/folders", FolderApi::router())
                    .with_state(FolderState::from_ref(self.api_state))
                    .nest("/library", LibraryApi::router())
                    .with_state(LibraryState::from_ref(self.api_state))
                    // .nest("/smartFeeds", SmartFeedApi::router())
                    // .with_state(SmartFeedState::from_ref(self.api_state))
                    .nest("/tags", TagApi::router())
                    .with_state(TagState::from_ref(self.api_state)),
            )
            .split_for_parts();

        openapi.info.title = "Colette API".to_owned();
        openapi.servers = Some(vec![Server::new(self.api_prefix)]);

        openapi.paths.paths = openapi
            .paths
            .paths
            .drain(RangeFull)
            .map(|(k, v)| (k.replace(self.api_prefix, ""), v))
            .collect();

        api.merge(Scalar::with_url(
            format!("{}/doc", self.api_prefix),
            openapi.clone(),
        ))
        .route(
            &format!("{}/openapi.json", self.api_prefix),
            routing::get(|| async move { openapi.to_pretty_json().unwrap() }),
        )
    }
}
