use chrono::{DateTime, Utc};
use colette_core::{
    Account, ApiKey, Bookmark, Collection, Feed, FeedEntry, Stream, Tag, User,
    api_key::ApiKeySearched,
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

pub struct FeedWithTagsAndCount {
    pub subscription: subscriptions::Model,
    pub feed: feeds::Model,
    pub tags: Vec<tags::Model>,
    pub unread_count: i64,
}

impl From<FeedWithTagsAndCount> for Feed {
    fn from(value: FeedWithTagsAndCount) -> Self {
        Self {
            id: value.subscription.id.parse().unwrap(),
            link: value.feed.link.parse().unwrap(),
            title: value.subscription.title,
            xml_url: value.feed.xml_url.and_then(|e| e.parse().ok()),
            user_id: value.subscription.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.subscription.created_at),
            updated_at: parse_timestamp(value.subscription.updated_at),
            tags: Some(value.tags.into_iter().map(Into::into).collect()),
            unread_count: Some(value.unread_count),
        }
    }
}

pub struct SubscriptionEntryWithFe {
    pub se: subscription_entries::Model,
    pub fe: feed_entries::Model,
}

impl From<SubscriptionEntryWithFe> for FeedEntry {
    fn from(value: SubscriptionEntryWithFe) -> Self {
        Self {
            id: value.se.id.parse().unwrap(),
            link: value.fe.link.parse().unwrap(),
            title: value.fe.title,
            published_at: parse_timestamp(value.fe.published_at).unwrap(),
            description: value.fe.description,
            author: value.fe.author,
            thumbnail_url: value.fe.thumbnail_url.and_then(|e| e.parse().ok()),
            has_read: value.se.has_read == 1,
            feed_id: value.se.subscription_id.parse().unwrap(),
            user_id: value.se.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.se.created_at),
            updated_at: parse_timestamp(value.se.updated_at),
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
