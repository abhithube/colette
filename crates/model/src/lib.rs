use chrono::{DateTime, Utc};
use colette_core::{
    Account, ApiKey, Bookmark, Collection, Feed, FeedEntry, Stream, Subscription,
    SubscriptionEntry, Tag, User, api_key::ApiKeySearched,
};
pub use entity::*;
use sea_orm::{Related, RelationDef, RelationTrait};

mod entity;

fn parse_timestamp(value: i32) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(value.into(), 0)
}

pub struct AccountWithUser {
    pub account: accounts::Model,
    pub user: users::Model,
}

impl From<AccountWithUser> for Account {
    fn from(value: AccountWithUser) -> Self {
        Self {
            id: value.user.id.parse().unwrap(),
            email: value.user.email,
            provider_id: value.account.provider_id,
            account_id: value.account.account_id,
            password_hash: value.account.password_hash,
        }
    }
}

impl From<api_keys::Model> for ApiKey {
    fn from(value: api_keys::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            preview: value.preview,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.created_at),
        }
    }
}

impl From<api_keys::Model> for ApiKeySearched {
    fn from(value: api_keys::Model) -> Self {
        Self {
            verification_hash: value.verification_hash,
            user_id: value.user_id.parse().unwrap(),
        }
    }
}

pub struct BookmarkWithTags {
    pub bookmark: bookmarks::Model,
    pub tags: Vec<tags::Model>,
}

impl From<BookmarkWithTags> for Bookmark {
    fn from(value: BookmarkWithTags) -> Self {
        Self {
            id: value.bookmark.id.parse().unwrap(),
            link: value.bookmark.link.parse().unwrap(),
            title: value.bookmark.title,
            thumbnail_url: value.bookmark.thumbnail_url.and_then(|e| e.parse().ok()),
            published_at: value.bookmark.published_at.and_then(parse_timestamp),
            author: value.bookmark.author,
            archived_path: value.bookmark.archived_path,
            user_id: value.bookmark.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.bookmark.created_at),
            updated_at: parse_timestamp(value.bookmark.updated_at),
            tags: Some(value.tags.into_iter().map(Into::into).collect()),
        }
    }
}

impl Related<tags::Entity> for bookmarks::Entity {
    fn to() -> RelationDef {
        bookmark_tags::Relation::Tags.def()
    }

    fn via() -> Option<RelationDef> {
        Some(bookmark_tags::Relation::Bookmarks.def().rev())
    }
}

impl From<collections::Model> for Collection {
    fn from(value: collections::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}

impl From<feeds::Model> for Feed {
    fn from(value: feeds::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            link: value.link.parse().unwrap(),
            xml_url: value.xml_url.and_then(|e| e.parse().ok()),
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at.and_then(parse_timestamp),
        }
    }
}

pub struct SubscriptionWithTagsAndCount {
    pub subscription: subscriptions::Model,
    pub feed: feeds::Model,
    pub tags: Vec<tags::Model>,
    pub unread_count: i64,
}

impl From<SubscriptionWithTagsAndCount> for Subscription {
    fn from(value: SubscriptionWithTagsAndCount) -> Self {
        Self {
            id: value.subscription.id.parse().unwrap(),
            feed: value.feed.into(),
            title: value.subscription.title,
            user_id: value.subscription.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.subscription.created_at),
            updated_at: parse_timestamp(value.subscription.updated_at),
            tags: Some(value.tags.into_iter().map(Into::into).collect()),
            unread_count: Some(value.unread_count),
        }
    }
}

impl Related<subscriptions::Entity> for feed_entries::Entity {
    fn to() -> RelationDef {
        feeds::Relation::Subscriptions.def()
    }

    fn via() -> Option<RelationDef> {
        Some(feeds::Relation::FeedEntries.def().rev())
    }
}

impl From<feed_entries::Model> for FeedEntry {
    fn from(value: feed_entries::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            link: value.link.parse().unwrap(),
            title: value.title,
            published_at: parse_timestamp(value.published_at).unwrap(),
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url.and_then(|e| e.parse().ok()),
            feed_id: value.feed_id.parse().unwrap(),
        }
    }
}

pub struct FeedEntryWithRead {
    pub fe: feed_entries::Model,
    pub subscription: subscriptions::Model,
    pub re: Option<read_entries::Model>,
}

impl From<FeedEntryWithRead> for SubscriptionEntry {
    fn from(value: FeedEntryWithRead) -> Self {
        Self {
            entry: value.fe.into(),
            has_read: value.re.is_some(),
            subscription_id: value.subscription.id.parse().unwrap(),
            user_id: value.subscription.user_id.parse().unwrap(),
        }
    }
}

impl From<streams::Model> for Stream {
    fn from(value: streams::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            filter: serde_json::from_str(&value.filter_raw).unwrap(),
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}

impl From<tags::Model> for Tag {
    fn from(value: tags::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            ..Default::default()
        }
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct TagWithCounts {
    id: String,
    title: String,
    user_id: String,
    created_at: i32,
    updated_at: i32,
    feed_count: i64,
    bookmark_count: i64,
}

impl From<TagWithCounts> for Tag {
    fn from(value: TagWithCounts) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            feed_count: Some(value.feed_count),
            bookmark_count: Some(value.bookmark_count),
        }
    }
}

impl Related<tags::Entity> for subscriptions::Entity {
    fn to() -> RelationDef {
        subscription_tags::Relation::Tags.def()
    }

    fn via() -> Option<RelationDef> {
        Some(subscription_tags::Relation::Subscriptions.def().rev())
    }
}

impl From<users::Model> for User {
    fn from(value: users::Model) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            email: value.email,
            display_name: value.display_name,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
