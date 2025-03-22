use std::sync::Arc;

use axum::extract::FromRef;
use colette_auth::AuthAdapter;
use colette_core::{
    api_key::ApiKeyService, backup::BackupService, bookmark::BookmarkService,
    collection::CollectionService, feed::FeedService, feed_entry::FeedEntryService,
    job::JobService, stream::StreamService, subscription::SubscriptionService,
    subscription_entry::SubscriptionEntryService, tag::TagService,
};
use torii::Torii;
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
pub mod subscription;
pub mod subscription_entry;
pub mod tag;

#[derive(Clone, FromRef)]
pub struct ApiState {
    pub auth: Arc<Torii<AuthAdapter>>,
    pub api_key_service: Arc<ApiKeyService>,
    pub backup_service: Arc<BackupService>,
    pub bookmark_service: Arc<BookmarkService>,
    pub collection_service: Arc<CollectionService>,
    pub feed_service: Arc<FeedService>,
    pub feed_entry_service: Arc<FeedEntryService>,
    pub job_service: Arc<JobService>,
    pub stream_service: Arc<StreamService>,
    pub subscription_service: Arc<SubscriptionService>,
    pub subscription_entry_service: Arc<SubscriptionEntryService>,
    pub tag_service: Arc<TagService>,
    pub image_base_url: Url,
}
