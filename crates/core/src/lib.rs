#![feature(bufreader_peek)]
pub use std::error::Error as StdError;

pub use auth::User;
pub use backup::Backup;
pub use bookmark::Bookmark;
pub use collection::Collection;
pub use feed::Feed;
pub use feed_entry::FeedEntry;
pub use subscription::Subscription;
pub use subscription_entry::SubscriptionEntry;
pub use tag::Tag;

pub mod auth;
pub mod backup;
pub mod bookmark;
pub mod collection;
pub mod common;
pub mod feed;
pub mod feed_entry;
pub mod filter;
pub mod job;
pub mod pagination;
pub mod subscription;
pub mod subscription_entry;
pub mod tag;

#[async_trait::async_trait]
pub trait Handler<C> {
    type Response;
    type Error: StdError;

    async fn handle(&self, cmd: C) -> Result<Self::Response, Self::Error>;
}
