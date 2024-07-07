use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct SelectManyParams<'a> {
    pub profile_id: &'a str,
    pub limit: i64,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub profile_feed_id: Option<&'a str>,
    pub has_read: Option<bool>,
}

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: String,
    pub profile_feed_id: &'a str,
    pub feed_entry_id: i64,
}
