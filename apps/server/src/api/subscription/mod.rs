use chrono::{DateTime, Utc};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use super::{ApiState, common::Paginated, feed::Feed, tag::Tag};

mod create_subscription;
mod delete_subscription;
mod get_subscription;
mod list_subscriptions;
mod mark_subscription_entry_as_read;
mod mark_subscription_entry_as_unread;
mod update_subscription;

pub const SUBSCRIPTIONS_TAG: &str = "Subscriptions";

#[derive(OpenApi)]
#[openapi(components(schemas(
    Subscription,
    Paginated<Subscription>,
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
    }
}

#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub id: Uuid,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    pub feed: Feed,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
    #[schema(nullable = false)]
    pub unread_count: Option<i64>,
}

impl From<colette_core::Subscription> for Subscription {
    fn from(value: colette_core::Subscription) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
            feed: value.feed.into(),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
            unread_count: value.unread_count,
        }
    }
}
