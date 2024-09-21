use chrono::{DateTime, Utc};
use colette_core::{Bookmark, Collection, Feed, FeedEntry, Profile, Tag, User};
pub use generated::*;
use sea_orm::{Related, RelationDef, RelationTrait};
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
            collection_id: value.pb.collection_id,
            tags: if value.tags.is_empty() {
                None
            } else {
                Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>())
            },
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialCollection {
    id: Uuid,
    title: String,
    parent_id: Option<Uuid>,
    bookmark_count: i64,
}

impl From<PartialCollection> for Collection {
    fn from(value: PartialCollection) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            bookmark_count: Some(value.bookmark_count),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PfWithFeedAndTagsAndUnreadCount {
    pub pf: profile_feed::Model,
    pub feed: feed::Model,
    pub tags: Vec<PartialFeedTag>,
    pub unread_count: i64,
}

impl From<PfWithFeedAndTagsAndUnreadCount> for Feed {
    fn from(value: PfWithFeedAndTagsAndUnreadCount) -> Self {
        Self {
            id: value.pf.id,
            link: value.feed.link,
            title: value.pf.title,
            pinned: value.pf.pinned,
            original_title: value.feed.title,
            url: value.feed.url,
            tags: if value.tags.is_empty() {
                None
            } else {
                Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>())
            },
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

impl From<tag::Model> for Tag {
    fn from(value: tag::Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialTag {
    id: Uuid,
    title: String,
    parent_id: Option<Uuid>,
    bookmark_count: i64,
    feed_count: i64,
}

impl From<PartialTag> for Tag {
    fn from(value: PartialTag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            bookmark_count: Some(value.bookmark_count),
            feed_count: Some(value.feed_count),
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialBookmarkTag {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub profile_bookmark_id: Uuid,
    pub level: i32,
}

impl From<PartialBookmarkTag> for Tag {
    fn from(value: PartialBookmarkTag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

#[derive(Clone, Debug, sea_orm::FromQueryResult)]
pub struct PartialFeedTag {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub profile_feed_id: Uuid,
    pub level: i32,
}

impl From<PartialFeedTag> for Tag {
    fn from(value: PartialFeedTag) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
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
