use std::io::BufRead;

use feed_rs::{
    model::{Entry, Feed, Link},
    parser,
};
use http::Response;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    extractor::{Error, Extractor},
    utils::{ExtractorQuery, TextSelector},
};

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

pub struct DefaultFeedExtractor {}

impl Extractor for DefaultFeedExtractor {
    type Extracted = ExtractedFeed;

    fn extract(
        &self,
        _url: &Url,
        resp: Response<Box<dyn BufRead>>,
    ) -> Result<ExtractedFeed, Error> {
        let feed = parser::parse(resp.into_body())
            .map(ExtractedFeed::from)
            .map_err(|e| Error(e.into()))?;

        Ok(feed)
    }
}

pub struct HtmlExtractor<'a> {
    options: FeedExtractorOptions<'a>,
}

impl Extractor for HtmlExtractor<'_> {
    type Extracted = ExtractedFeed;

    fn extract(
        &self,
        _url: &Url,
        resp: Response<Box<dyn BufRead>>,
    ) -> Result<ExtractedFeed, Error> {
        let mut body = resp.into_body();

        let mut bytes: Vec<u8> = vec![];
        body.read(&mut bytes).map_err(|e| Error(e.into()))?;

        let raw = String::from_utf8_lossy(&bytes);
        let html = Html::parse_document(&raw);

        let entries = html
            .select(&self.options.feed_entries_selector)
            .map(|element| ExtractedFeedEntry {
                link: element.select_text(&self.options.feed_entry_link_queries),
                title: element.select_text(&self.options.feed_entry_title_queries),
                published: element.select_text(&self.options.feed_entry_published_queries),
                description: element.select_text(&self.options.feed_entry_description_queries),
                author: element.select_text(&self.options.feed_entry_author_queries),
                thumbnail: element.select_text(&self.options.feed_entry_thumbnail_queries),
            })
            .collect();

        let feed = ExtractedFeed {
            link: html.select_text(&self.options.feed_link_queries),
            title: html.select_text(&self.options.feed_title_queries),
            entries,
        };

        Ok(feed)
    }
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
