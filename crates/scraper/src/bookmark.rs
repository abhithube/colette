use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Read},
};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use http::{Request, Response};
use scraper::{Html, Selector};
use url::Url;

use crate::{
    utils::{ExtractorQuery, Node, TextSelector},
    DownloaderError, DownloaderPlugin, ExtractorError, PostprocessorError, Scraper,
};

#[derive(Clone, Debug)]
pub struct BookmarkExtractorOptions<'a> {
    pub title_queries: Vec<ExtractorQuery<'a>>,
    pub published_queries: Vec<ExtractorQuery<'a>>,
    pub author_queries: Vec<ExtractorQuery<'a>>,
    pub thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

impl<'a> Default for BookmarkExtractorOptions<'a> {
    fn default() -> Self {
        Self {
            title_queries: vec![
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/VideoObject'] > [itemprop='name']",
                    )
                    .unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[property='og:title']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[name='twitter:title']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[name='title']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(Selector::parse("title").unwrap(), Node::Text),
            ],
            published_queries: vec![
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/VideoObject'] > [itemprop='datePublished']",
                    )
                    .unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/VideoObject'] > [itemprop='uploadDate']",
                    )
                    .unwrap(),
                    Node::Attr("content"),
                ),
            ],
            author_queries: vec![ExtractorQuery::new(
                Selector::parse("[itemtype='http://schema.org/Person'] > [itemprop='name']")
                    .unwrap(),
                Node::Attr("content"),
            )],
            thumbnail_queries: vec![
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/ImageObject'] > [itemprop='url']",
                    )
                    .unwrap(),
                    Node::Attr("href"),
                ),
                ExtractorQuery::new(
                    Selector::parse("[itemprop='thumbnailUrl']").unwrap(),
                    Node::Attr("href"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[property='og:image']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[name='twitter:image']").unwrap(),
                    Node::Attr("content"),
                ),
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedBookmark {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub published: Option<String>,
    pub author: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ProcessedBookmark {
    pub title: String,
    pub thumbnail: Option<Url>,
    pub published: Option<DateTime<Utc>>,
    pub author: Option<String>,
}

impl TryFrom<ExtractedBookmark> for ProcessedBookmark {
    type Error = PostprocessorError;

    fn try_from(mut value: ExtractedBookmark) -> Result<Self, Self::Error> {
        let Some(title) = value.title else {
            return Err(PostprocessorError(anyhow!(
                "could not process bookmark title"
            )));
        };

        if let Some(t) = &value.thumbnail {
            if t.starts_with("//") {
                value.thumbnail = Some(format!("https:{t}"));
            }
        }

        let bookmark = ProcessedBookmark {
            title,
            thumbnail: value.thumbnail.as_ref().and_then(|e| Url::parse(e).ok()),
            published: value.published.as_ref().and_then(|e| {
                DateTime::parse_from_rfc3339(e)
                    .ok()
                    .or(DateTime::parse_from_rfc2822(e).ok())
                    .map(|f| f.to_utc())
            }),
            author: value.author,
        };

        Ok(bookmark)
    }
}

pub type BookmarkPostprocessorPlugin =
    fn(url: &Url, extracted: &mut ExtractedBookmark) -> Result<(), PostprocessorError>;

pub struct BookmarkPlugin<'a> {
    pub downloader: DownloaderPlugin,
    pub extractor: BookmarkExtractorOptions<'a>,
    pub postprocessor: BookmarkPostprocessorPlugin,
}

impl Default for BookmarkPlugin<'_> {
    fn default() -> Self {
        Self {
            downloader: |url| {
                Request::get(url.as_str())
                    .body(())
                    .map(|e| e.into_parts().0)
                    .map_err(|e| DownloaderError(e.into()))
            },
            extractor: BookmarkExtractorOptions::default(),
            postprocessor: |_url, _extracted| Ok(()),
        }
    }
}

pub struct DefaultBookmarkScraper<'a> {
    registry: HashMap<&'static str, BookmarkPlugin<'a>>,
    default_plugin: BookmarkPlugin<'a>,
}

impl<'a> DefaultBookmarkScraper<'a> {
    pub fn new(registry: HashMap<&'static str, BookmarkPlugin<'a>>) -> Self {
        Self {
            registry,
            default_plugin: BookmarkPlugin::default(),
        }
    }
}

#[async_trait::async_trait]
impl Scraper<ProcessedBookmark> for DefaultBookmarkScraper<'_> {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, crate::Error> {
        let host = url.host_str().ok_or(crate::Error::Parse)?;

        let plugin = self.registry.get(host).unwrap_or(&self.default_plugin);

        let parts = (plugin.downloader)(url)?;
        let req: ureq::Request = parts.into();
        let resp = req.call().map_err(|e| DownloaderError(e.into()))?;

        let resp: Response<Box<dyn Read + Send + Sync>> = resp.into();
        let resp = resp.map(|e| Box::new(BufReader::new(e)) as Box<dyn BufRead>);

        let mut body = resp.into_body();

        let mut bytes: Vec<u8> = vec![];
        body.read(&mut bytes)
            .map_err(|e| ExtractorError(e.into()))?;

        let raw = String::from_utf8_lossy(&bytes);
        let html = Html::parse_document(&raw);

        let mut extracted = ExtractedBookmark {
            title: html.select_text(&plugin.extractor.title_queries),
            thumbnail: html.select_text(&plugin.extractor.thumbnail_queries),
            published: html.select_text(&plugin.extractor.published_queries),
            author: html.select_text(&plugin.extractor.author_queries),
        };

        (plugin.postprocessor)(url, &mut extracted)?;
        let processed = extracted.try_into()?;

        Ok(processed)
    }
}
