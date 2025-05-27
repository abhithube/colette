use axum::{Router, routing};
use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use uuid::Uuid;

use super::{ApiState, common::Paginated, feed::Feed, tag::Tag};

mod create_subscription;
mod delete_subscription;
mod export_subscriptions;
mod get_subscription;
mod import_subscriptions;
mod link_subscription_tags;
mod list_subscriptions;
mod mark_subscription_entry_as_read;
mod mark_subscription_entry_as_unread;
mod update_subscription;

const SUBSCRIPTIONS_TAG: &str = "Subscriptions";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Subscription, SubscriptionDetails, Paginated<SubscriptionDetails>, create_subscription::SubscriptionCreate, update_subscription::SubscriptionUpdate, link_subscription_tags::LinkSubscriptionTags)),
    paths(list_subscriptions::handler, create_subscription::handler, get_subscription::handler, update_subscription::handler, delete_subscription::handler, link_subscription_tags::handler, mark_subscription_entry_as_read::handler, mark_subscription_entry_as_unread::handler, import_subscriptions::handler, export_subscriptions::handler)
)]
pub(crate) struct SubscriptionApi;

impl SubscriptionApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new()
            .route("/", routing::get(list_subscriptions::handler))
            .route("/", routing::post(create_subscription::handler))
            .route("/{id}", routing::get(get_subscription::handler))
            .route("/{id}", routing::patch(update_subscription::handler))
            .route("/{id}", routing::delete(delete_subscription::handler))
            .route(
                "/{id}/linkTags",
                routing::post(link_subscription_tags::handler),
            )
            .route(
                "/{sid}/entries/{eid}/markAsRead",
                routing::post(mark_subscription_entry_as_read::handler),
            )
            .route(
                "/{sid}/entries/{eid}/markAsUnread",
                routing::post(mark_subscription_entry_as_unread::handler),
            )
            .route("/import", routing::post(import_subscriptions::handler))
            .route("/export", routing::post(export_subscriptions::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Subscription {
    id: Uuid,
    title: String,
    #[schema(required)]
    description: Option<String>,
    feed_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct SubscriptionDetails {
    subscription: Subscription,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed: Option<Feed>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<Tag>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    unread_count: Option<i64>,
}

impl From<colette_core::Subscription> for Subscription {
    fn from(value: colette_core::Subscription) -> Self {
        Self {
            id: value.id,
            title: value.title,
            description: value.description,
            feed_id: value.feed_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<colette_core::Subscription> for SubscriptionDetails {
    fn from(value: colette_core::Subscription) -> Self {
        let feed = value.feed.clone().map(Into::into);
        let tags = value
            .tags
            .clone()
            .map(|e| e.into_iter().map(Into::into).collect());
        let unread_count = value.unread_count;

        Self {
            subscription: value.into(),
            feed,
            tags,
            unread_count,
        }
    }
}
