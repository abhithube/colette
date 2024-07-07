use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Entry {
    pub id: String,
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: String,
}

#[derive(Debug)]
pub struct ListEntriesParams {
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<String>,
    pub has_read: Option<bool>,
}
