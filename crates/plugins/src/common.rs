use core::str;

use bytes::Bytes;
use colette_scraper::{
    ExtractorQuery, TextSelector,
    bookmark::{BookmarkError, ExtractedBookmark},
    feed::{ExtractedFeed, ExtractedFeedEntry, FeedError},
};
use scraper::{Html, Selector};

#[derive(Debug, Clone, Default)]
pub struct FeedExtractorOptions {
    pub feed_link_queries: Vec<ExtractorQuery>,
    pub feed_title_queries: Vec<ExtractorQuery>,
    pub feed_description_queries: Vec<ExtractorQuery>,
    pub feed_refreshed_queries: Vec<ExtractorQuery>,
    pub feed_entries_selectors: Vec<Selector>,
    pub feed_entry_link_queries: Vec<ExtractorQuery>,
    pub feed_entry_title_queries: Vec<ExtractorQuery>,
    pub feed_entry_published_queries: Vec<ExtractorQuery>,
    pub feed_entry_description_queries: Vec<ExtractorQuery>,
    pub feed_entry_author_queries: Vec<ExtractorQuery>,
    pub feed_entry_thumbnail_queries: Vec<ExtractorQuery>,
}

#[derive(Debug, Clone, Default)]
pub struct FeedExtractor {
    options: FeedExtractorOptions,
}

impl FeedExtractor {
    pub fn new(options: FeedExtractorOptions) -> Self {
        Self { options }
    }

    pub fn extract(&self, body: Bytes) -> Result<ExtractedFeed, FeedError> {
        let html = Html::parse_document(str::from_utf8(&body)?);

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
            description: html.select_text(&self.options.feed_description_queries),
            refreshed: html.select_text(&self.options.feed_refreshed_queries),
            entries,
        };

        Ok(feed)
    }
}

#[derive(Debug, Clone, Default)]
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

    pub fn extract(&self, body: Bytes) -> Result<ExtractedBookmark, BookmarkError> {
        let html = Html::parse_document(str::from_utf8(&body)?);

        let bookmark = ExtractedBookmark {
            title: html.select_text(&self.options.title_queries),
            thumbnail: html.select_text(&self.options.thumbnail_queries),
            published: html.select_text(&self.options.published_queries),
            author: html.select_text(&self.options.author_queries),
        };

        Ok(bookmark)
    }
}
