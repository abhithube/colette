use colette_core::common::{FindManyParams, FindOneParams};
use uuid::Uuid;

pub mod bookmark_tags;
pub mod bookmarks;
pub mod collections;
pub mod entries;
pub mod feed_entries;
pub mod feeds;
pub mod profile_feed_entries;
pub mod profile_feed_tags;
pub mod profile_feeds;
pub mod profiles;
pub mod tags;
pub mod users;

#[derive(Clone, Debug)]
pub struct SelectManyParams {
    pub profile_id: Uuid,
}

impl<'a> From<&'a FindManyParams> for SelectManyParams {
    fn from(value: &'a FindManyParams) -> Self {
        Self {
            profile_id: value.profile_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SelectByIdParams {
    pub id: Uuid,
    pub profile_id: Uuid,
}

impl<'a> From<&'a FindOneParams> for SelectByIdParams {
    fn from(value: &'a FindOneParams) -> Self {
        Self {
            id: value.id,
            profile_id: value.profile_id,
        }
    }
}
