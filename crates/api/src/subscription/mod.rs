use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, common::Paginated, feed::Feed, tag::Tag};

mod create_subscription;
mod delete_subscription;
mod export_subscriptions;
mod get_subscription;
mod import_subscriptions;
mod list_subscriptions;
mod mark_subscription_entry_as_read;
mod mark_subscription_entry_as_unread;
mod update_subscription;

pub const SUBSCRIPTIONS_TAG: &str = "Subscriptions";

#[derive(OpenApi)]
#[openapi(components(schemas(
    Subscription,
    SubscriptionDetails,
    Paginated<SubscriptionDetails>,
    create_subscription::SubscriptionCreate,
    update_subscription::SubscriptionUpdate,
)))]
pub struct SubscriptionApi;

impl SubscriptionApi {
    pub fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(SubscriptionApi::openapi())
            .routes(routes!(
                list_subscriptions::handler,
                create_subscription::handler
            ))
            .routes(routes!(
                get_subscription::handler,
                update_subscription::handler,
                delete_subscription::handler
            ))
            .routes(routes!(mark_subscription_entry_as_read::handler))
            .routes(routes!(mark_subscription_entry_as_unread::handler))
            .routes(routes!(import_subscriptions::handler))
            .routes(routes!(export_subscriptions::handler))
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: Uuid,
    pub title: String,
    pub feed_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDetails {
    pub subscription: Subscription,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed: Option<Feed>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unread_count: Option<i64>,
}

impl From<colette_core::Subscription> for Subscription {
    fn from(value: colette_core::Subscription) -> Self {
        Self {
            id: value.id,
            title: value.title,
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
