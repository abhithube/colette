use chrono::{DateTime, Utc};
use colette_core::entries::EntryFindManyParams;

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a str,
    pub limit: i64,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub profile_feed_id: Option<&'a str>,
    pub has_read: Option<bool>,
}

impl<'a> From<&'a EntryFindManyParams> for SelectManyParams<'a> {
    fn from(value: &'a EntryFindManyParams) -> Self {
        Self {
            profile_id: &value.profile_id,
            limit: value.limit,
            published_at: value.published_at.as_ref(),
            profile_feed_id: value.feed_id.as_deref(),
            has_read: value.has_read,
        }
    }
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub profile_feed_id: &'a str,
    pub feed_entry_id: i64,
}
