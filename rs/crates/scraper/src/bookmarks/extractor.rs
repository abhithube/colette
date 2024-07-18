use anyhow::anyhow;
use colette_core::{
    bookmarks::{BookmarkExtractorOptions, ExtractedBookmark},
    utils::scraper::{ExtractError, Extractor},
};
use libxml::{parser::Parser, xpath::Context};

use crate::{
    base_extractor_options, microdata_extractor_options, open_graph_extractor_options,
    twitter_extractor_options, utils::Xpath,
};

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
        let document = Parser::default_html()
            .parse_string(raw)
            .map_err(|e| ExtractError(e.into()))?;

        let mut context = Context::new(&document)
            .map_err(|_| ExtractError(anyhow!("couldn't create xpath context from document")))?;

        let bookmark = ExtractedBookmark {
            title: context.find_first_content(&self.options.title_expr, None),
            thumbnail: context.find_first_content(&self.options.thumbnail_expr, None),
            published: context.find_first_content(&self.options.published_expr, None),
            author: context.find_first_content(&self.options.author_expr, None),
        };

        Ok(bookmark)
    }
}

fn merge(options_vec: Vec<BookmarkExtractorOptions>) -> BookmarkExtractorOptions {
    fn merge_field<'a>(fields: &[Vec<&'a str>]) -> Vec<&'a str> {
        fields.iter().flat_map(|v| v.iter().cloned()).collect()
    }

    BookmarkExtractorOptions {
        title_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.title_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        published_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.published_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        author_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.author_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        thumbnail_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.thumbnail_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
    }
}
