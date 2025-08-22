use axum::{Router, routing};
use chrono::{DateTime, Utc};
use colette_core::subscription;
use url::Url;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{ApiState, pagination::Paginated, tag::Tag};

mod create_subscription;
mod delete_subscription;
mod export_subscriptions;
mod get_subscription;
mod import_subscriptions;
mod link_subscription_tags;
mod list_subscriptions;
mod update_subscription;

const SUBSCRIPTIONS_TAG: &str = "Subscriptions";

#[derive(OpenApi)]
#[openapi(
    components(schemas(Subscription, Paginated<Subscription>, create_subscription::SubscriptionCreate, update_subscription::SubscriptionUpdate, link_subscription_tags::LinkSubscriptionTags)),
    paths(list_subscriptions::handler, create_subscription::handler, get_subscription::handler, update_subscription::handler, delete_subscription::handler, link_subscription_tags::handler, import_subscriptions::handler, export_subscriptions::handler)
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
            .route("/import", routing::post(import_subscriptions::handler))
            .route("/export", routing::post(export_subscriptions::handler))
    }
}

/// User subscription to an RSS feed
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct Subscription {
    /// Unique identifier of the subscription
    id: Uuid,
    /// Feed source URL
    source_url: Url,
    /// URL of the webpage the feed links to
    link: Url,
    /// Human-readable name of the subscription
    title: String,
    /// Description of the subscription
    #[schema(required)]
    description: Option<String>,
    /// Linked tags
    tags: Vec<Tag>,
    /// Count of unread subscription entries associated with the subscription
    unread_count: i64,
    /// Timestamp at which the subscription was created
    created_at: DateTime<Utc>,
    /// Timestamp at which the subscription was modified
    updated_at: DateTime<Utc>,
}

impl From<subscription::SubscriptionDto> for Subscription {
    fn from(value: subscription::SubscriptionDto) -> Self {
        Self {
            id: value.id,
            source_url: value.source_url,
            link: value.link,
            title: value.title,
            description: value.description,
            tags: value.tags.into_iter().map(Into::into).collect(),
            unread_count: value.unread_count,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
