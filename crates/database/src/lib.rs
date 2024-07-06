use colette_core::common;

pub mod entries;
pub mod feed_entries;
pub mod feeds;
pub mod profile_feed_entries;
pub mod profile_feeds;
pub mod profiles;
pub mod users;

#[derive(Debug)]
pub struct FindOneParams<'a> {
    pub id: &'a str,
    pub profile_id: &'a str,
}

impl<'a> From<&'a common::FindOneParams> for FindOneParams<'a> {
    fn from(value: &'a common::FindOneParams) -> Self {
        Self {
            id: &value.id,
            profile_id: &value.profile_id,
        }
    }
}
