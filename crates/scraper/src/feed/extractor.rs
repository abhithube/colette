use bytes::Bytes;
use scraper::{Html, Selector};

use crate::{
    utils::{ExtractorQuery, TextSelector},
    ExtractorError,
};

use super::{ExtractedFeed, ExtractedFeedEntry};

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

#[derive(Debug, Clone, Default)]
pub struct FeedExtractor<'a> {
    options: FeedExtractorOptions<'a>,
}

impl<'a> FeedExtractor<'a> {
    pub fn new(options: FeedExtractorOptions<'a>) -> Self {
        Self { options }
    }

    pub fn extract(&self, body: Bytes) -> Result<ExtractedFeed, ExtractorError> {
        let raw = String::from_utf8(body.into()).map_err(|e| ExtractorError(e.into()))?;

        let html = Html::parse_document(&raw);

        let entries = self
            .options
            .feed_entries_selectors
            .iter()
            .find_map(|e| {
                let entries = html
                    .select(e)
                    .map(|element| ExtractedFeedEntry {
                        link: element.select_text(&self.options.feed_entry_link_queries),
                        title: element.select_text(&self.options.feed_entry_title_queries),
                        published: element.select_text(&self.options.feed_entry_published_queries),
                        description: element
                            .select_text(&self.options.feed_entry_description_queries),
                        author: element.select_text(&self.options.feed_entry_author_queries),
                        thumbnail: element.select_text(&self.options.feed_entry_thumbnail_queries),
                    })
                    .collect::<Vec<_>>();

                if entries.is_empty() {
                    None
                } else {
                    Some(entries)
                }
            })
            .unwrap_or_default();

        let feed = ExtractedFeed {
            link: html.select_text(&self.options.feed_link_queries),
            title: html.select_text(&self.options.feed_title_queries),
            entries,
        };

        Ok(feed)
    }
}