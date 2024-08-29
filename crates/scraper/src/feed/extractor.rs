use std::str::FromStr;

use anyhow::anyhow;
use atom_syndication::Feed;
use http::Response;
use rss::Channel;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    extractor::{Error, Extractor},
    utils::{ExtractorQuery, TextSelector},
};

#[derive(Clone, Debug, Default)]
pub struct FeedExtractorOptions<'a> {
    pub feed_link_queries: Vec<ExtractorQuery<'a>>,
    pub feed_title_queries: Vec<ExtractorQuery<'a>>,
    pub feed_entries_selector: &'a str,
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
    type T = ExtractedFeed;

    fn extract(&self, _url: &Url, resp: Response<String>) -> Result<ExtractedFeed, Error> {
        let (parts, body) = resp.into_parts();

        let content_type = parts
            .headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|e| e.to_str().ok());

        let feed = if content_type.map_or(false, |e| e.contains("application/atom+xml"))
            || body.contains("<feed")
        {
            Feed::from_str(&body)
                .map(ExtractedFeed::from)
                .map_err(|e| Error(e.into()))
        } else if content_type.map_or(false, |e| e.contains("application/rss+xml"))
            || body.contains("<rss")
        {
            Channel::from_str(&body)
                .map(ExtractedFeed::from)
                .map_err(|e| Error(e.into()))
        } else {
            Err(Error(anyhow!("couldn't find extractor for feed URL")))
        }?;

        Ok(feed)
    }
}

pub struct HtmlExtractor<'a> {
    options: FeedExtractorOptions<'a>,
}

impl Extractor for HtmlExtractor<'_> {
    type T = ExtractedFeed;

    fn extract(&self, _url: &Url, resp: Response<String>) -> Result<ExtractedFeed, Error> {
        let raw = resp.into_body();
        let html = Html::parse_document(&raw);

        let entries = html
            .select(&Selector::parse(self.options.feed_entries_selector).unwrap())
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
