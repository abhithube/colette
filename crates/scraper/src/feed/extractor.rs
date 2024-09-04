use feed_rs::model::{Entry, Feed, Link};
use scraper::Selector;

use crate::utils::ExtractorQuery;

#[derive(Clone, Debug)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selector: Selector,
    pub feed_entry_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_published_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_description_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_author_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entry_thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedFeedEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedFeedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}

impl From<Feed> for ExtractedFeed {
    fn from(value: Feed) -> Self {
        Self {
            link: parse_atom_link(value.links),
            title: value.title.map(|e| e.content),
            entries: value
                .entries
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<Entry> for ExtractedFeedEntry {
    fn from(mut value: Entry) -> Self {
        let mut title = value.title.map(|e| e.content);
        let mut description = value.summary.map(|e| e.content);
        let mut thumbnail = Option::<String>::None;

        if !value.media.is_empty() {
            let mut media = value.media.swap_remove(0);
            if let Some(t) = media.title.map(|e| e.content) {
                title = Some(t);
            }
            if let Some(d) = media.description.map(|e| e.content) {
                description = Some(d);
            }
            if !media.thumbnails.is_empty() {
                thumbnail = Some(media.thumbnails.swap_remove(0).image.uri);
            } else if !media.content.is_empty() {
                let content = media.content.swap_remove(0);
                if let Some(content_type) = content.content_type {
                    if content_type.ty().as_str() == "image" {
                        if let Some(url) = content.url {
                            thumbnail = Some(url.into())
                        }
                    }
                }
            }
        }

        Self {
            link: parse_atom_link(value.links),
            title,
            published: value.published.map(|e| e.to_rfc3339()),
            description,
            author: Some(
                value
                    .authors
                    .into_iter()
                    .map(|e| e.name)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            thumbnail,
        }
    }
}

fn parse_atom_link(links: Vec<Link>) -> Option<String> {
    links.into_iter().find_map(|l| match l.rel.as_deref() {
        Some("alternate") | None => Some(l.href),
        _ => None,
    })
}
