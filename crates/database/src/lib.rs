use colette_core::common;
use uuid::Uuid;

pub mod bookmarks;
pub mod collections;
pub mod entries;
pub mod feed_entries;
pub mod feeds;
pub mod profile_feed_entries;
pub mod profile_feeds;
pub mod profiles;
pub mod users;

#[derive(Clone, Debug)]
pub struct SelectByIdParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
}

impl<'a> From<&'a common::FindOneParams> for SelectByIdParams<'a> {
    fn from(value: &'a common::FindOneParams) -> Self {
        Self {
            id: &value.id,
            profile_id: &value.profile_id,
        }
    }
}
