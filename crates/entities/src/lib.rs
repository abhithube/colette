use chrono::{DateTime, Utc};
use colette_core::{Bookmark, Entry, Feed, Profile, Tag, User};
pub use generated::*;
use sea_orm::{Linked, Related, RelationDef, RelationTrait};
use uuid::Uuid;

mod generated;

#[derive(Clone, Debug)]
pub struct PfeWithEntry {
    pub pfe: profile_feed_entry::Model,
    pub entry: entry::Model,
}

impl From<PfeWithEntry> for Entry {
    fn from(value: PfeWithEntry) -> Self {
        Self {
            id: value.pfe.id,
            link: value.entry.link,
            title: value.entry.title,
            published_at: value.entry.published_at.map(DateTime::<Utc>::from),
            description: value.entry.description,
            author: value.entry.author,
            thumbnail_url: value.entry.thumbnail_url,
            has_read: value.pfe.has_read,
            feed_id: value.pfe.profile_feed_id,
        }
    }
}

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
            tags: Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>()),
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
            tags: Some(value.tags.into_iter().map(Tag::from).collect::<Vec<_>>()),
            unread_count: Some(value.unread_count),
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

#[derive(Debug)]
pub struct ProfileFeedEntryToEntry;

impl Linked for ProfileFeedEntryToEntry {
    type FromEntity = profile_feed_entry::Entity;

    type ToEntity = entry::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            profile_feed_entry::Relation::FeedEntry.def(),
            feed_entry::Relation::Entry.def(),
        ]
    }
}
