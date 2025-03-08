use chrono::{DateTime, Utc};
use colette_core::{
    Account, ApiKey, Bookmark, Collection, Feed, FeedEntry, Stream, Subscription,
    SubscriptionEntry, Tag, User,
};
pub use entity::*;

mod entity;

fn parse_timestamp(value: i32) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(value.into(), 0)
}

#[derive(sea_orm::FromQueryResult)]
pub struct AccountRow {
    pub email: String,
    pub provider_id: String,
    pub account_id: String,
    pub password_hash: Option<String>,
    pub user_id: String,
}

impl From<AccountRow> for Account {
    fn from(value: AccountRow) -> Self {
        Self {
            email: value.email,
            provider_id: value.provider_id,
            account_id: value.account_id,
            password_hash: value.password_hash,
            id: value.user_id.parse().unwrap(),
        }
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct ApiKeyRow {
    pub id: String,
    pub title: String,
    pub preview: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<ApiKeyRow> for ApiKey {
    fn from(value: ApiKeyRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            preview: value.preview,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct BookmarkRow {
    pub id: String,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<i32>,
    pub archived_path: Option<String>,
    pub author: Option<String>,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

#[derive(sea_orm::FromQueryResult)]
pub struct BookmarkTagRow {
    pub bookmark_id: String,
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<BookmarkTagRow> for Tag {
    fn from(value: BookmarkTagRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            ..Default::default()
        }
    }
}

pub struct BookmarkRowWithTagRows {
    pub bookmark: BookmarkRow,
    pub tags: Option<Vec<BookmarkTagRow>>,
}

impl From<BookmarkRowWithTagRows> for Bookmark {
    fn from(value: BookmarkRowWithTagRows) -> Self {
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
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct CollectionRow {
    pub id: String,
    pub title: String,
    pub filter_raw: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<CollectionRow> for Collection {
    fn from(value: CollectionRow) -> Self {
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

#[derive(sea_orm::FromQueryResult)]
pub struct FeedEntryRow {
    pub id: String,
    pub link: String,
    pub title: String,
    pub published_at: i32,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub feed_id: String,
}

impl From<FeedEntryRow> for FeedEntry {
    fn from(value: FeedEntryRow) -> Self {
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

#[derive(sea_orm::FromQueryResult)]
pub struct FeedRow {
    pub id: String,
    pub link: String,
    pub xml_url: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<i32>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
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

#[derive(sea_orm::FromQueryResult)]
pub struct StreamRow {
    pub id: String,
    pub title: String,
    pub filter_raw: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<StreamRow> for Stream {
    fn from(value: StreamRow) -> Self {
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

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriptionEntryRow {
    pub id: String,
    pub link: String,
    pub title: String,
    pub published_at: i32,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub feed_id: String,

    pub subscription_id: String,
    pub user_id: String,
    pub has_read: bool,
}

impl From<SubscriptionEntryRow> for SubscriptionEntry {
    fn from(value: SubscriptionEntryRow) -> Self {
        Self {
            entry: FeedEntryRow {
                id: value.id,
                link: value.link,
                title: value.title,
                published_at: value.published_at,
                description: value.description,
                author: value.author,
                thumbnail_url: value.thumbnail_url,
                feed_id: value.feed_id,
            }
            .into(),
            subscription_id: value.subscription_id.parse().unwrap(),
            user_id: value.user_id.parse().unwrap(),
            has_read: value.has_read,
        }
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriptionRow {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,

    pub feed_id: String,
    pub link: String,
    pub xml_url: Option<String>,
    pub feed_title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<i32>,
}

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriptionTagRow {
    pub subscription_id: String,
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<SubscriptionTagRow> for Tag {
    fn from(value: SubscriptionTagRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            ..Default::default()
        }
    }
}

pub struct SubscriptionWithTagsAndCount {
    pub subscription: SubscriptionRow,
    pub tags: Option<Vec<SubscriptionTagRow>>,
    pub unread_count: Option<i64>,
}

impl From<SubscriptionWithTagsAndCount> for Subscription {
    fn from(value: SubscriptionWithTagsAndCount) -> Self {
        Self {
            id: value.subscription.id.parse().unwrap(),
            title: value.subscription.title,
            user_id: value.subscription.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.subscription.created_at),
            updated_at: parse_timestamp(value.subscription.updated_at),
            feed: FeedRow {
                id: value.subscription.feed_id,
                link: value.subscription.link,
                xml_url: value.subscription.xml_url,
                title: value.subscription.feed_title,
                description: value.subscription.description,
                refreshed_at: value.subscription.refreshed_at,
            }
            .into(),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
            unread_count: value.unread_count,
        }
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct TagWithCounts {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
    pub feed_count: i64,
    pub bookmark_count: i64,
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

#[derive(sea_orm::FromQueryResult)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            email: value.email,
            display_name: value.display_name,
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
        }
    }
}
