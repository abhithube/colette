use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub profile_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub bookmark_count: Option<i64>,
}
