use chrono::{DateTime, Utc};
use url::Url;
use uuid::Uuid;

use crate::{bookmark::BookmarkDto, subscription::SubscriptionDto, tag::TagDto};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Backup {
    pub bookmarks: Vec<BackupBookmark>,
    pub subscriptions: Vec<BackupSubscription>,
    pub tags: Vec<BackupTag>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupBookmark {
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_path: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<BackupTag>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<BookmarkDto> for BackupBookmark {
    fn from(value: BookmarkDto) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            tags: value.tags.into_iter().map(Into::into).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupSubscription {
    pub id: Uuid,
    pub source_url: Url,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<BackupTag>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<SubscriptionDto> for BackupSubscription {
    fn from(value: SubscriptionDto) -> Self {
        Self {
            id: value.id,
            source_url: value.source_url,
            title: value.title,
            description: value.description,
            tags: value.tags.into_iter().map(Into::into).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupTag {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TagDto> for BackupTag {
    fn from(value: TagDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
