use chrono::{DateTime, Utc};
use colette_core::{Bookmark, Collection, Feed, FeedEntry, Folder, Profile, Tag, User};
pub use generated::*;
use sea_orm::{Related, RelationDef, RelationTrait};
use uuid::Uuid;

mod generated;

#[derive(Clone, Debug)]
pub struct PbWithBookmarkAndTags {
    pub pb: profile_bookmark::Model,
    pub bookmark: bookmark::Model,
    pub tags: Vec<tag::Model>,
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
            collection_id: value.pb.collection_id,
            tags: Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>()),
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialCollection {
    id: Uuid,
    title: String,
    folder_id: Option<Uuid>,
    bookmark_count: i64,
}

impl From<PartialCollection> for Collection {
    fn from(value: PartialCollection) -> Self {
        Self {
            id: value.id,
            title: value.title,
            folder_id: value.folder_id,
            bookmark_count: Some(value.bookmark_count),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PfWithFeedAndTagsAndUnreadCount {
    pub pf: profile_feed::Model,
    pub feed: feed::Model,
    pub tags: Vec<tag::Model>,
    pub unread_count: i64,
}

impl From<PfWithFeedAndTagsAndUnreadCount> for Feed {
    fn from(value: PfWithFeedAndTagsAndUnreadCount) -> Self {
        Self {
            id: value.pf.id,
            link: value.feed.link,
            title: value.pf.title,
            original_title: value.feed.title,
            url: value.feed.url,
            folder_id: value.pf.folder_id,
            tags: Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>()),
            unread_count: Some(value.unread_count),
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
            published_at: value.fe.published_at.map(DateTime::<Utc>::from),
            description: value.fe.description,
            author: value.fe.author,
            thumbnail_url: value.fe.thumbnail_url,
            has_read: value.pfe.has_read,
            feed_id: value.pfe.profile_feed_id,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialFolder {
    id: Uuid,
    title: String,
    parent_id: Option<Uuid>,
    collection_count: i64,
    feed_count: i64,
}

impl From<PartialFolder> for Folder {
    fn from(value: PartialFolder) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            collection_count: Some(value.collection_count),
            feed_count: Some(value.feed_count),
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

impl From<tag::Model> for Tag {
    fn from(value: tag::Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialTag {
    id: Uuid,
    title: String,
    bookmark_count: i64,
    feed_count: i64,
}

impl From<PartialTag> for Tag {
    fn from(value: PartialTag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: Some(value.bookmark_count),
            feed_count: Some(value.feed_count),
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

impl Related<tag::Entity> for profile_bookmark::Entity {
    fn to() -> RelationDef {
        profile_bookmark_tag::Relation::Tag.def()
    }

    fn via() -> Option<RelationDef> {
        Some(profile_bookmark::Relation::ProfileBookmarkTag.def())
    }
}

impl Related<tag::Entity> for profile_feed::Entity {
    fn to() -> RelationDef {
        profile_feed_tag::Relation::Tag.def()
    }

    fn via() -> Option<RelationDef> {
        Some(profile_feed::Relation::ProfileFeedTag.def())
    }
}
