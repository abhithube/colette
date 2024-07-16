use chrono::{DateTime, Utc};
use colette_core::feeds::ProcessedEntry;

#[derive(Debug)]
pub struct InsertParams<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub description: Option<&'a str>,
    pub author: Option<&'a str>,
    pub thumbnail_url: Option<&'a str>,
}

impl<'a> From<&'a ProcessedEntry> for InsertParams<'a> {
    fn from(value: &'a ProcessedEntry) -> Self {
        Self {
            link: value.link.as_str(),
            title: &value.title,
            published_at: value.published.as_ref(),
            description: value.description.as_deref(),
            author: value.author.as_deref(),
            thumbnail_url: value.thumbnail.as_ref().map(|e| e.as_str()),
        }
    }
}
