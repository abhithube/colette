use colette_core::{
    bookmarks::ExtractedBookmark,
    utils::scraper::{ExtractError, Extractor},
};
use scraper::Html;

use crate::{
    base_extractor_options,
    feeds::{Item, TextSelector},
    microdata_extractor_options, open_graph_extractor_options, twitter_extractor_options,
};

pub struct BookmarkExtractorOptions<'a> {
    pub title_selectors: Vec<Item<'a>>,
    pub published_selectors: Vec<Item<'a>>,
    pub author_selectors: Vec<Item<'a>>,
    pub thumbnail_selectors: Vec<Item<'a>>,
}

pub struct DefaultBookmarkExtractor<'a> {
    options: BookmarkExtractorOptions<'a>,
}

impl<'a> DefaultBookmarkExtractor<'a> {
    pub fn new(options: Option<BookmarkExtractorOptions<'a>>) -> Self {
        Self {
            options: options.unwrap_or(merge(vec![
                open_graph_extractor_options(),
                twitter_extractor_options(),
                microdata_extractor_options(),
                base_extractor_options(),
            ])),
        }
    }
}

impl Extractor<ExtractedBookmark> for DefaultBookmarkExtractor<'_> {
    fn extract(&self, _url: &str, raw: &str) -> Result<ExtractedBookmark, ExtractError> {
        let html = Html::parse_document(raw);

        let bookmark = ExtractedBookmark {
            title: html.select_text(&self.options.title_selectors),
            thumbnail: html.select_text(&self.options.thumbnail_selectors),
            published: html.select_text(&self.options.published_selectors),
            author: html.select_text(&self.options.author_selectors),
        };

        Ok(bookmark)
    }
}

fn merge(options_vec: Vec<BookmarkExtractorOptions>) -> BookmarkExtractorOptions {
    fn merge_field<'a>(fields: &[Vec<Item<'a>>]) -> Vec<Item<'a>> {
        fields.iter().flat_map(|v| v.iter().cloned()).collect()
    }

    BookmarkExtractorOptions {
        title_selectors: merge_field(
            &options_vec
                .iter()
                .map(|e| e.title_selectors.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        thumbnail_selectors: merge_field(
            &options_vec
                .iter()
                .map(|e| e.thumbnail_selectors.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        published_selectors: merge_field(
            &options_vec
                .iter()
                .map(|e| e.published_selectors.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        author_selectors: merge_field(
            &options_vec
                .iter()
                .map(|e| e.author_selectors.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
    }
}
