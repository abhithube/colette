use core::str;
use std::str::Utf8Error;

use chrono::{DateTime, Utc};
use colette_feed::{
    Feed,
    atom::{AtomEntry, AtomFeed, AtomLink, AtomRel},
    rss::{RssFeed, RssItem},
};
use url::Url;

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

#[async_trait::async_trait]
pub trait FeedScraper: Send + Sync + 'static {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, ScraperError>;
}

#[derive(Debug, Clone, Default)]
pub struct ExtractedFeed {
    pub link: Option<String>,
    pub title: Option<String>,
    pub entries: Vec<ExtractedFeedEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct ExtractedFeedEntry {
    pub link: Option<String>,
    pub title: Option<String>,
    pub published: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProcessedFeed {
    pub link: Url,
    pub title: String,
    pub entries: Vec<ProcessedFeedEntry>,
}

#[derive(Debug, Clone)]
pub struct ProcessedFeedEntry {
    pub link: Url,
    pub title: String,
    pub published: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail: Option<Url>,
}

impl TryFrom<ExtractedFeed> for ProcessedFeed {
    type Error = PostprocessorError;

    fn try_from(value: ExtractedFeed) -> Result<Self, Self::Error> {
        let Some(Ok(link)) = value.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessorError::Link);
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError::Title);
        };
        if title.is_empty() {
            return Err(PostprocessorError::Title);
        }

        let mut entries: Vec<ProcessedFeedEntry> = Vec::new();

        for entry in value.entries.into_iter() {
            entries.push(entry.try_into()?);
        }

        let feed = Self {
            link,
            title: title.trim().to_owned(),
            entries,
        };

        Ok(feed)
    }
}

impl TryFrom<ExtractedFeedEntry> for ProcessedFeedEntry {
    type Error = PostprocessorError;

    fn try_from(value: ExtractedFeedEntry) -> Result<Self, Self::Error> {
        let Some(Ok(link)) = value.link.as_ref().map(|e| Url::parse(e)) else {
            return Err(PostprocessorError::Link);
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError::Title);
        };
        if title.is_empty() {
            return Err(PostprocessorError::Title);
        }

        let Some(published) = value.published.as_ref().and_then(|e| {
            DateTime::parse_from_rfc3339(e.trim())
                .ok()
                .or(DateTime::parse_from_rfc2822(e).ok())
                .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                .map(|f| f.to_utc())
        }) else {
            return Err(PostprocessorError::Published);
        };
        let thumbnail = value
            .thumbnail
            .as_ref()
            .and_then(|e| Url::parse(e.trim()).ok());

        let entry = Self {
            link,
            title: title.trim().to_owned(),
            published,
            description: value.description.and_then(|e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e.trim().to_owned())
                }
            }),
            author: value.author.and_then(|e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e.trim().to_owned())
                }
            }),
            thumbnail,
        };

        Ok(entry)
    }
}

impl From<Feed> for ExtractedFeed {
    fn from(value: Feed) -> Self {
        match value {
            Feed::Atom(atom) => atom.into(),
            Feed::Rss(rss) => rss.into(),
        }
    }
}

impl From<AtomFeed> for ExtractedFeed {
    fn from(value: AtomFeed) -> Self {
        Self {
            link: parse_atom_link(value.link),
            title: Some(value.title.text),
            entries: value
                .entry
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<AtomEntry> for ExtractedFeedEntry {
    fn from(value: AtomEntry) -> Self {
        let mut title = value.title.text;
        let mut description = value.summary.or(value.content).map(|e| e.text);
        let mut thumbnail = Option::<String>::None;

        if let Some(extension) = value.extension {
            if let Some(mut media_group) = extension.media_group {
                if let Some(media_title) = media_group.media_title {
                    title = media_title;
                }
                if media_group.media_description.is_some() {
                    description = media_group.media_description;
                }

                if !media_group.media_thumbnail.is_empty() {
                    let media_thumbnail = media_group.media_thumbnail.swap_remove(0);
                    thumbnail = Some(media_thumbnail.url);
                }
            }
        }

        Self {
            link: parse_atom_link(value.link),
            title: Some(title),
            published: value.published,
            description,
            author: Some(
                value
                    .author
                    .into_iter()
                    .map(|e| e.name)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            thumbnail,
        }
    }
}

fn parse_atom_link(links: Vec<AtomLink>) -> Option<String> {
    links.into_iter().find_map(|l| match l.rel {
        AtomRel::Alternate => Some(l.href),
        _ => None,
    })
}

impl From<RssFeed> for ExtractedFeed {
    fn from(value: RssFeed) -> Self {
        Self {
            link: Some(value.channel.link),
            title: Some(value.channel.title),
            entries: value
                .channel
                .item
                .into_iter()
                .map(ExtractedFeedEntry::from)
                .collect(),
        }
    }
}

impl From<RssItem> for ExtractedFeedEntry {
    fn from(value: RssItem) -> Self {
        Self {
            link: Some(value.link),
            title: Some(value.title),
            published: value.pub_date,
            description: Some(value.description),
            author: value.author,
            thumbnail: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScraperError {
    #[error("document type not supported")]
    Unsupported,

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error(transparent)]
    Postprocess(#[from] PostprocessorError),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] Utf8Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Feed(#[from] colette_feed::Error),

    #[error(transparent)]
    Meta(#[from] colette_meta::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum PostprocessorError {
    #[error("could not process link")]
    Link,

    #[error("could not process title")]
    Title,

    #[error("could not process published date")]
    Published,
}
