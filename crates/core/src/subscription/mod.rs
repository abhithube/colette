use chrono::{DateTime, Utc};
pub use create_subscription_handler::*;
pub use delete_subscription_handler::*;
pub use export_subscriptions_handler::*;
pub use get_subscription_handler::*;
pub use import_subscriptions_handler::*;
pub use link_subscription_tags_handler::*;
pub use list_subscriptions_handler::*;
pub use subscription_repository::*;
pub use update_subscription_handler::*;
use uuid::Uuid;

use crate::{Feed, Tag, pagination::Cursor};

mod create_subscription_handler;
mod delete_subscription_handler;
mod export_subscriptions_handler;
mod get_subscription_handler;
mod import_subscriptions_handler;
mod link_subscription_tags_handler;
mod list_subscriptions_handler;
mod subscription_repository;
mod update_subscription_handler;

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub feed_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub feed: Feed,
    pub tags: Option<Vec<Tag>>,
    pub unread_count: Option<i64>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SubscriptionCursor {
    pub title: String,
    pub id: Uuid,
}

impl Cursor for Subscription {
    type Data = SubscriptionCursor;

    fn to_cursor(&self) -> Self::Data {
        Self::Data {
            title: self.title.clone(),
            id: self.id,
        }
    }
}
