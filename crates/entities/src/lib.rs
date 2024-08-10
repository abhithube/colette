use chrono::{DateTime, Utc};
use colette_core::{Entry, Profile, User};
pub use generated::*;
use sea_orm::{Linked, RelationDef, RelationTrait};

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

impl From<user::Model> for User {
    fn from(value: user::Model) -> Self {
        Self {
            id: value.id,
            email: value.email,
            password: value.password,
        }
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
