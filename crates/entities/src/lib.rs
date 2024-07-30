use chrono::{DateTime, FixedOffset, Utc};
use colette_core::{Bookmark, Entry, Feed, Profile, Tag, User};
pub use generated::*;
use sea_orm::{prelude::Uuid, ColumnTrait, Linked, RelationDef, RelationTrait};

mod generated;

pub struct BookmarkWithTags(pub bookmark::Model, pub Option<Vec<tag::Model>>);

impl From<BookmarkWithTags> for Bookmark {
    fn from(BookmarkWithTags(bookmark, tags): BookmarkWithTags) -> Self {
        Self {
            id: bookmark.id,
            link: bookmark.link,
            title: bookmark.title,
            published_at: bookmark.published_at.map(DateTime::<Utc>::from),
            author: bookmark.author,
            thumbnail_url: bookmark.thumbnail_url,
            profile_id: bookmark.profile_id,
            created_at: bookmark.created_at.into(),
            updated_at: bookmark.updated_at.into(),
            tags: tags.map(|e| e.into_iter().map(Tag::from).collect()),
        }
    }
}

pub struct PfeWithEntry(pub profile_feed_entry::Model, pub entry::Model);

impl From<PfeWithEntry> for Entry {
    fn from(PfeWithEntry(pfe, entry): PfeWithEntry) -> Self {
        Self {
            id: pfe.id,
            link: entry.link,
            title: entry.title,
            published_at: entry.published_at.map(DateTime::<Utc>::from),
            description: entry.description,
            author: entry.author,
            thumbnail_url: entry.thumbnail_url,
            feed_id: pfe.profile_feed_id,
            has_read: pfe.has_read,
        }
    }
}

#[derive(sea_orm::DerivePartialModel, sea_orm::FromQueryResult)]
#[sea_orm(entity = "profile_feed::Entity")]
pub struct PartialFeed {
    id: Uuid,
    #[sea_orm(from_expr = "feed::Column::Title")]
    title: String,
    #[sea_orm(from_expr = "feed::Column::Link")]
    link: String,
    #[sea_orm(from_expr = "feed::Column::Url")]
    url: Option<String>,
    profile_id: Uuid,
    created_at: DateTime<FixedOffset>,
    updated_at: DateTime<FixedOffset>,
    #[sea_orm(from_expr = "profile_feed_entry::Column::Id.count()")]
    unread_count: Option<i64>,
}

impl From<PartialFeed> for Feed {
    fn from(value: PartialFeed) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            url: value.url,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
            unread_count: value.unread_count,
        }
    }
}

impl From<profile::Model> for Profile {
    fn from(value: profile::Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            user_id: value.user_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

impl From<tag::Model> for Tag {
    fn from(value: tag::Model) -> Self {
        Self {
            id: value.id,
            title: value.title,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

impl From<user::Model> for User {
    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
        }
    }
}

#[derive(Debug)]
pub struct BookmarkToTag;

impl Linked for BookmarkToTag {
    type FromEntity = bookmark::Entity;
    type ToEntity = tag::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            bookmark::Relation::BookmarkTag.def(),
            bookmark_tag::Relation::Tag.def(),
        ]
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
