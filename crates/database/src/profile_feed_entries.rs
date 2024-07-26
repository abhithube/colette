use chrono::{DateTime, Utc};
use colette_core::entries::EntriesFindManyParams;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: Uuid,
    pub limit: i64,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub profile_feed_id: Option<Uuid>,
    pub has_read: Option<bool>,
}

impl<'a> From<&'a EntriesFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a EntriesFindManyParams) -> Self {
        Self {
            profile_id: value.profile_id,
            limit: value.limit,
            published_at: value.published_at.as_ref(),
            profile_feed_id: value.feed_id,
            has_read: value.has_read,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpdateParams {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub has_read: Option<bool>,
}
