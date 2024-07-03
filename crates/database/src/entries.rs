use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct InsertData<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub description: Option<&'a str>,
    pub author: Option<&'a str>,
    pub thumbnail_url: Option<&'a str>,
}
