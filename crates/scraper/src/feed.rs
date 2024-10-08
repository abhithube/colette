use std::{
    collections::HashMap,
    io::{BufReader, Read},
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use colette_feed::{
    atom::{AtomEntry, AtomFeed, AtomLink, AtomRel},
    rss::{RssFeed, RssItem},
    Feed,
};
use http::{request::Builder, Request, Response};
use scraper::{Html, Selector};
use url::Url;

use crate::{
    utils::{ExtractorQuery, TextSelector},
    DownloaderError, Error, ExtractorError, PostprocessorError,
};

const RFC2822_WITHOUT_COMMA: &str = "%a %d %b %Y %H:%M:%S %z";

#[derive(Clone, Debug, Default)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selectors: Vec<Selector>,
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

#[derive(Clone, Debug)]
pub struct ProcessedFeed {
    pub link: Url,
    pub title: String,
    pub entries: Vec<ProcessedFeedEntry>,
}

#[derive(Clone, Debug)]
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
            return Err(PostprocessorError(anyhow!("could not process feed link")));
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError(anyhow!("could not process feed title")));
        };
        if title.is_empty() {
            return Err(PostprocessorError(anyhow!("could not process feed title")));
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
            return Err(PostprocessorError(anyhow!("could not process value link")));
        };
        let Some(title) = value.title else {
            return Err(PostprocessorError(anyhow!("could not process value title")));
        };
        if title.is_empty() {
            return Err(PostprocessorError(anyhow!("could not process value title")));
        }

        let Some(published) = value.published.as_ref().and_then(|e| {
            DateTime::parse_from_rfc3339(e.trim())
                .ok()
                .or(DateTime::parse_from_rfc2822(e).ok())
                .or(DateTime::parse_from_str(e, RFC2822_WITHOUT_COMMA).ok())
                .map(|f| f.to_utc())
        }) else {
            return Err(PostprocessorError(anyhow!(
                "could not process entry publish date"
            )));
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

pub trait FeedScraper: Send + Sync {
    fn before_download(&self, url: &mut Url) -> Builder {
        Request::get(url.as_str())
    }

    fn download(
        &self,
        builder: Builder,
    ) -> Result<Response<Box<dyn Read + Send + Sync>>, DownloaderError> {
        let req: ureq::Request = builder
            .try_into()
            .map_err(|e: http::Error| DownloaderError(e.into()))?;

        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        Ok(resp.into())
    }

    fn before_extract(&self) -> Option<FeedExtractorOptions> {
        None
    }

    fn extract(
        &self,
        url: &Url,
        resp: Response<Box<dyn Read + Send + Sync>>,
    ) -> Result<ExtractedFeed, ExtractorError> {
        match self.before_extract() {
            Some(options) => {
                let mut body = resp.into_body();
                let mut raw = String::new();
                body.read_to_string(&mut raw)
                    .map_err(|e| ExtractorError(e.into()))?;

                let html = Html::parse_document(&raw);

                let entries = options
                    .feed_entries_selectors
                    .iter()
                    .find_map(|e| {
                        let entries = html
                            .select(e)
                            .map(|element| ExtractedFeedEntry {
                                link: element.select_text(&options.feed_entry_link_queries),
                                title: element.select_text(&options.feed_entry_title_queries),
                                published: element
                                    .select_text(&options.feed_entry_published_queries),
                                description: element
                                    .select_text(&options.feed_entry_description_queries),
                                author: element.select_text(&options.feed_entry_author_queries),
                                thumbnail: element
                                    .select_text(&options.feed_entry_thumbnail_queries),
                            })
                            .collect::<Vec<_>>();

                        if entries.is_empty() {
                            None
                        } else {
                            Some(entries)
                        }
                    })
                    .unwrap_or_default();

                let mut feed = ExtractedFeed {
                    link: html.select_text(&options.feed_link_queries),
                    title: html.select_text(&options.feed_title_queries),
                    entries,
                };
                if feed.link.is_none() {
                    feed.link = Some(url.to_string());
                }

                Ok(feed)
            }
            None => colette_feed::from_reader(BufReader::new(resp.into_body()))
                .map(ExtractedFeed::from)
                .map_err(|e| e.into()),
        }
    }

    #[allow(unused_variables)]
    fn before_postprocess(
        &self,
        url: &Url,
        feed: &mut ExtractedFeed,
    ) -> Result<(), PostprocessorError> {
        Ok(())
    }

    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let builder = self.before_download(url);
        let resp = self.download(builder)?;
        let mut feed = self.extract(url, resp)?;
        self.before_postprocess(url, &mut feed)?;

        Ok(feed.try_into()?)
    }
}

#[derive(Default)]
pub struct FeedPluginRegistry {
    plugins: HashMap<&'static str, Box<dyn FeedScraper>>,
}

impl FeedPluginRegistry {
    pub fn new(plugins: HashMap<&'static str, Box<dyn FeedScraper>>) -> Self {
        Self { plugins }
    }
}

impl FeedScraper for FeedPluginRegistry {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, Error> {
        let host = url.host_str().ok_or(Error::Parse)?;

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url),
            None => {
                let builder = self.before_download(url);
                let resp = self.download(builder)?;
                let mut feed = self.extract(url, resp)?;
                self.before_postprocess(url, &mut feed)?;

                Ok(feed.try_into()?)
            }
        }
    }
}
