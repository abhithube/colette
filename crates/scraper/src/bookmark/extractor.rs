use bytes::Bytes;
use scraper::Html;

use super::ExtractedBookmark;
use crate::{
    utils::{ExtractorQuery, TextSelector},
    ExtractorError,
};

#[derive(Clone, Debug, Default)]
pub struct BookmarkExtractorOptions {
    pub title_queries: Vec<ExtractorQuery>,
    pub published_queries: Vec<ExtractorQuery>,
    pub author_queries: Vec<ExtractorQuery>,
    pub thumbnail_queries: Vec<ExtractorQuery>,
}

#[derive(Debug, Clone, Default)]
pub struct BookmarkExtractor {
    options: BookmarkExtractorOptions,
}

impl BookmarkExtractor {
    pub fn new(options: BookmarkExtractorOptions) -> Self {
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
