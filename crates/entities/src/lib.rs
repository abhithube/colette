use chrono::{DateTime, Utc};
use colette_core::{Bookmark, Profile, Tag, User};
pub use generated::*;

mod generated;

impl From<bookmark::Model> for Bookmark {
    fn from(value: bookmark::Model) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at.map(DateTime::<Utc>::from),
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            profile_id: value.profile_id,
            created_at: value.created_at.into(),
            updated_at: value.updated_at.into(),
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
