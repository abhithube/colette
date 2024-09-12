use std::{collections::HashMap, io::Read};

use anyhow::anyhow;
use chrono::{DateTime, Utc};
use colette_meta::{
    open_graph,
    schema_org::{SchemaObject, SchemaObjectOrValue},
};
use http::{request::Builder, Request, Response};
use scraper::Html;
use url::Url;

use crate::{
    utils::{ExtractorQuery, TextSelector},
    DownloaderError, Error, ExtractorError, PostprocessorError,
};

#[derive(Clone, Debug, Default)]
pub struct BookmarkExtractorOptions<'a> {
    pub title_queries: Vec<ExtractorQuery<'a>>,
    pub published_queries: Vec<ExtractorQuery<'a>>,
    pub author_queries: Vec<ExtractorQuery<'a>>,
    pub thumbnail_queries: Vec<ExtractorQuery<'a>>,
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

    fn before_extract(&self) -> Option<BookmarkExtractorOptions> {
        None
    }

    #[allow(unused_variables)]
    fn extract(
        &self,
        url: &Url,
        resp: Response<Box<dyn Read + Send + Sync>>,
    ) -> Result<ExtractedBookmark, ExtractorError> {
        let mut body = resp.into_body();

        match self.before_extract() {
            Some(options) => {
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
            None => {
                let metadata = colette_meta::parse_metadata(body);

                let mut bookmark = ExtractedBookmark {
                    title: metadata.basic.title,
                    thumbnail: None,
                    published: None,
                    author: metadata.basic.author,
                };

                if let Some(mut og) = metadata.open_graph {
                    if !og.title.is_empty() {
                        bookmark.title = Some(og.title);
                    }
                    bookmark.thumbnail = Some(og.images.swap_remove(0).url);

                    if let open_graph::Type::Article(article) = og.r#type {
                        bookmark.published = article.published_time;
                    }
                }

                if bookmark.title.is_none()
                    || bookmark.thumbnail.is_none()
                    || bookmark.published.is_none()
                    || bookmark.author.is_none()
                {
                    for schema in metadata.schema_org {
                        if let SchemaObjectOrValue::SchemaObject(schema) = schema {
                            match schema {
                                SchemaObject::Article(article) => {
                                    bookmark.title = bookmark.title.or(article.name);
                                    bookmark.thumbnail = bookmark
                                        .thumbnail
                                        .or(article.thumbnail_url)
                                        .or(article.thumbnail.and_then(|e| e.url));
                                    bookmark.published =
                                        bookmark.published.or(article.date_published);
                                }
                                SchemaObject::WebPage(webpage) => {
                                    bookmark.title = bookmark.title.or(webpage.name);
                                    bookmark.thumbnail = bookmark
                                        .thumbnail
                                        .or(webpage.thumbnail_url)
                                        .or(webpage.thumbnail.and_then(|e| e.url));
                                    bookmark.published =
                                        bookmark.published.or(webpage.date_published);
                                }
                                SchemaObject::ImageObject(image) => {
                                    bookmark.thumbnail = bookmark.thumbnail.or(image.url);
                                }
                                SchemaObject::VideoObject(video) => {
                                    bookmark.title = bookmark.title.or(video.name);
                                    bookmark.thumbnail = bookmark
                                        .thumbnail
                                        .or(video.thumbnail_url)
                                        .or(video.thumbnail.and_then(|e| e.url));
                                    bookmark.published =
                                        bookmark.published.or(video.date_published);
                                }
                                SchemaObject::Person(person) => {
                                    bookmark.author = bookmark.author.or(person.name);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                Ok(bookmark)
            }
        }
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
