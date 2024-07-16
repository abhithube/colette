use chrono::{DateTime, Utc};
use colette_core::entries::{EntryFindManyParams, EntryUpdateData};
use uuid::Uuid;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct UpdateData {
    pub has_read: Option<bool>,
}

impl<'a> From<&'a EntryUpdateData> for UpdateData {
    fn from(value: &'a EntryUpdateData) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}
