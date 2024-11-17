use std::collections::HashMap;

use anyhow::anyhow;
use bytes::{Buf, Bytes};
use chrono::{DateTime, Utc};
use colette_meta::{
    open_graph,
    schema_org::{SchemaObject, SchemaObjectOrValue},
};
use dyn_clone::DynClone;
use scraper::Html;
use url::Url;

use crate::{
    utils::{ExtractorQuery, TextSelector},
    Downloader, Error, ExtractorError, PostprocessorError,
};

const RFC3339_WITH_MILLI: &str = "%Y-%m-%dT%H:%M:%S%.3f%z";
const RFC3339_WITH_MICRO: &str = "%Y-%m-%dT%H:%M:%S%.6f%z";

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
                    .or(DateTime::parse_from_str(e, RFC3339_WITH_MILLI).ok())
                    .or(DateTime::parse_from_str(e, RFC3339_WITH_MICRO).ok())
                    .map(|f| f.to_utc())
            }),
            author: value.author,
        };

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
pub trait BookmarkScraper: Send + Sync + DynClone {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, Error>;
}

dyn_clone::clone_trait_object!(BookmarkScraper);

#[derive(Debug, Clone, Default)]
pub struct BookmarkExtractor<'a> {
    options: BookmarkExtractorOptions<'a>,
}

impl<'a> BookmarkExtractor<'a> {
    pub fn new(options: BookmarkExtractorOptions<'a>) -> Self {
        Self { options }
    }

    pub fn extract(&self, body: Bytes) -> Result<ExtractedBookmark, ExtractorError> {
        let raw = String::from_utf8(body.into()).map_err(|e| ExtractorError(e.into()))?;

        let html = Html::parse_document(&raw);

        let bookmark = ExtractedBookmark {
            title: html.select_text(&self.options.title_queries),
            thumbnail: html.select_text(&self.options.thumbnail_queries),
            published: html.select_text(&self.options.published_queries),
            author: html.select_text(&self.options.author_queries),
        };

        Ok(bookmark)
    }
}

#[derive(Clone)]
pub struct BookmarkPluginRegistry {
    plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
    downloader: Box<dyn Downloader>,
}

impl BookmarkPluginRegistry {
    pub fn new(
        plugins: HashMap<&'static str, Box<dyn BookmarkScraper>>,
        downloader: Box<dyn Downloader>,
    ) -> Self {
        Self {
            plugins,
            downloader,
        }
    }
}

#[async_trait::async_trait]
impl BookmarkScraper for BookmarkPluginRegistry {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, Error> {
        let host = url.host_str().ok_or(Error::Parse)?;

        match self.plugins.get(host) {
            Some(plugin) => plugin.scrape(url).await,
            None => {
                let body = self.downloader.download(url).await?;

                let metadata =
                    colette_meta::parse_metadata(body.reader()).map_err(ExtractorError)?;

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
                                    bookmark.author =
                                        bookmark.author.or(article.author.and_then(|e| e.name));
                                }
                                SchemaObject::WebPage(webpage) => {
                                    bookmark.title = bookmark.title.or(webpage.name);
                                    bookmark.thumbnail = bookmark
                                        .thumbnail
                                        .or(webpage.thumbnail_url)
                                        .or(webpage.thumbnail.and_then(|e| e.url));
                                    bookmark.published =
                                        bookmark.published.or(webpage.date_published);
                                    bookmark.author =
                                        bookmark.author.or(webpage.author.and_then(|e| e.name));
                                }
                                SchemaObject::VideoObject(video) => {
                                    bookmark.title = bookmark.title.or(video.name);
                                    bookmark.thumbnail = bookmark
                                        .thumbnail
                                        .or(video.thumbnail_url)
                                        .or(video.thumbnail.and_then(|e| e.url));
                                    bookmark.published =
                                        bookmark.published.or(video.date_published);
                                    bookmark.author =
                                        bookmark.author.or(video.author.and_then(|e| e.name));
                                }
                                SchemaObject::WebSite(website) => {
                                    bookmark.title = bookmark.title.or(website.name);
                                    bookmark.thumbnail = bookmark
                                        .thumbnail
                                        .or(website.thumbnail_url)
                                        .or(website.thumbnail.and_then(|e| e.url));
                                    bookmark.published =
                                        bookmark.published.or(website.date_published);
                                    bookmark.author =
                                        bookmark.author.or(website.author.and_then(|e| e.name));
                                }
                                SchemaObject::ImageObject(image) => {
                                    bookmark.thumbnail = bookmark.thumbnail.or(image.url);
                                }
                                SchemaObject::Person(person) => {
                                    bookmark.author = bookmark.author.or(person.name);
                                }
                            }
                        }
                    }
                }

                Ok(bookmark.try_into()?)
            }
        }
    }
}
