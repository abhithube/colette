use std::{collections::HashMap, io::Read};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use http::{request::Builder, Request, Response};
use scraper::{Html, Selector};
use url::Url;

use crate::{
    utils::{ExtractorQuery, Node, TextSelector},
    DownloaderError, Error, ExtractorError, PostprocessorError,
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

pub trait BookmarkScraper: Send + Sync {
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

    fn before_extract(&self) -> BookmarkExtractorOptions {
        BookmarkExtractorOptions::default()
    }

    #[allow(unused_variables)]
    fn extract(
        &self,
        url: &Url,
        resp: Response<Box<dyn Read + Send + Sync>>,
    ) -> Result<ExtractedBookmark, ExtractorError> {
        let options = self.before_extract();

        let mut body = resp.into_body();
        let mut raw = String::new();
        body.read_to_string(&mut raw)
            .map_err(|e| ExtractorError(e.into()))?;

        let html = Html::parse_document(&raw);

        let bookmark = ExtractedBookmark {
            title: html.select_text(&options.title_queries),
            thumbnail: html.select_text(&options.thumbnail_queries),
            published: html.select_text(&options.published_queries),
            author: html.select_text(&options.author_queries),
        };

        Ok(bookmark)
    }

    #[allow(unused_variables)]
    fn before_postprocess(
        &self,
        url: &Url,
        bookmark: &mut ExtractedBookmark,
    ) -> Result<(), PostprocessorError> {
        Ok(())
    }

    fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, Error> {
        let builder = self.before_download(url);
        let resp = self.download(builder)?;
        let mut feed = self.extract(url, resp)?;
        self.before_postprocess(url, &mut feed)?;

        Ok(feed.try_into()?)
    }
}

#[derive(Default)]
pub struct BookmarkPluginRegistry {
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
}

impl BookmarkPluginRegistry {
    pub fn new(plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>) -> Self {
        Self { plugins }
    }
}

impl BookmarkScraper for BookmarkPluginRegistry {
    fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, Error> {
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
