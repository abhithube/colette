use chrono::{DateTime, Utc};
use serde::Serialize;
use url::Url;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub id: String,
    pub title: String,
    pub link: String,
    pub url: Option<String>,
    pub custom_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub unread_count: Option<i64>,
}

#[derive(Debug)]
pub struct ExtractorOptions {
    pub feed_link_expr: Option<&'static str>,
    pub feed_title_expr: &'static str,
    pub feed_entries_expr: &'static str,
    pub entry_link_expr: &'static str,
    pub entry_title_expr: &'static str,
    pub entry_published_expr: Option<&'static str>,
    pub entry_description_expr: Option<&'static str>,
    pub entry_author_expr: Option<&'static str>,
    pub entry_thumbnail_expr: Option<&'static str>,
}

#[derive(Debug)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedEntry>,
}

#[derive(Debug)]
pub struct ExtractedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug)]
pub struct ProcessedFeed<'a> {
    pub link: Url,
    pub title: &'a str,
    pub entries: Vec<ProcessedEntry<'a>>,
}

#[derive(Debug)]
pub struct ProcessedEntry<'a> {
    pub link: Url,
    pub title: &'a str,
    pub published: Option<DateTime<Utc>>,
    pub description: Option<&'a str>,
    pub author: Option<&'a str>,
    pub thumbnail: Option<Url>,
}
