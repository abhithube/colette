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
