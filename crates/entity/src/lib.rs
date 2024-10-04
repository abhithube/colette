use chrono::{DateTime, Utc};
use colette_core::{Bookmark, FeedEntry, Profile, SmartFeed, Tag, User};
pub use generated::*;
use uuid::Uuid;

mod generated;

#[derive(Clone, Debug)]
pub struct PbWithBookmarkAndTags {
    pub pb: profile_bookmark::Model,
    pub bookmark: bookmark::Model,
    pub tags: Vec<PartialBookmarkTag>,
}

impl From<PbWithBookmarkAndTags> for Bookmark {
    fn from(value: PbWithBookmarkAndTags) -> Self {
        Self {
            id: value.pb.id,
            link: value.bookmark.link,
            title: value.bookmark.title,
            thumbnail_url: value.bookmark.thumbnail_url,
            published_at: value.bookmark.published_at.map(DateTime::<Utc>::from),
            author: value.bookmark.author,
            sort_index: value.pb.sort_index as u32,
            tags: if value.tags.is_empty() {
                None
            } else {
                Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>())
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct PfeWithFe {
    pub pfe: profile_feed_entry::Model,
    pub fe: feed_entry::Model,
}

impl From<PfeWithFe> for FeedEntry {
    fn from(value: PfeWithFe) -> Self {
        Self {
            id: value.pfe.id,
            link: value.fe.link,
            title: value.fe.title,
            published_at: value.fe.published_at.into(),
            description: value.fe.description,
            author: value.fe.author,
            thumbnail_url: value.fe.thumbnail_url,
            has_read: value.pfe.has_read,
            feed_id: value.pfe.profile_feed_id,
        }
    }
}

impl From<profile::Model> for Profile {
    fn from(value: profile::Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            is_default: value.is_default,
            user_id: value.user_id,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialSmartFeed {
    pub id: Uuid,
    pub title: String,
    pub unread_count: i64,
}

impl From<PartialSmartFeed> for SmartFeed {
    fn from(value: PartialSmartFeed) -> Self {
        Self {
            id: value.id,
            title: value.title,
            unread_count: Some(value.unread_count),
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialBookmarkTag {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub profile_bookmark_id: Uuid,
    pub depth: i32,
    pub direct: Option<bool>,
}

impl From<PartialBookmarkTag> for Tag {
    fn from(value: PartialBookmarkTag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            depth: value.depth,
            direct: value.direct,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

impl From<user::Model> for User {
    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
        }
    }
}
