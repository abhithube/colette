use std::sync::Arc;

use axum::extract::FromRef;
use colette_core::{
    api_key::ApiKeyService, auth::AuthService, backup::BackupService, bookmark::BookmarkService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    stream::StreamService, tag::TagService,
};
use url::Url;

pub mod api_key;
pub mod auth;
pub mod backup;
pub mod bookmark;
pub mod collection;
pub mod common;
pub mod feed;
pub mod feed_entry;
pub mod stream;
pub mod tag;

#[derive(Clone, FromRef)]
pub struct ApiState {
    pub api_key_service: Arc<ApiKeyService>,
    pub auth_service: Arc<AuthService>,
    pub backup_service: Arc<BackupService>,
    pub bookmark_service: Arc<BookmarkService>,
    pub collection_service: Arc<CollectionService>,
    pub feed_service: Arc<FeedService>,
    pub feed_entry_service: Arc<FeedEntryService>,
    pub stream_service: Arc<StreamService>,
    pub tag_service: Arc<TagService>,
    pub image_base_url: Url,
}
