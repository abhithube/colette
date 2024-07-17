use chrono::{DateTime, Utc};
use colette_core::entries::EntryFindManyParams;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a Uuid,
    pub limit: i64,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub profile_feed_id: Option<&'a Uuid>,
    pub has_read: Option<bool>,
}

impl<'a> From<&'a EntryFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a EntryFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
            limit: value.limit,
            published_at: value.published_at.as_ref(),
            profile_feed_id: value.feed_id.as_ref(),
            has_read: value.has_read,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateParams<'a> {
    pub id: &'a Uuid,
    pub profile_id: &'a Uuid,
    pub has_read: Option<bool>,
}
