use anyhow::anyhow;
use colette_core::{
    bookmarks::{BookmarkExtractorOptions, ExtractedBookmark},
    utils::scraper::{ExtractError, Extractor},
};
use libxml::{parser::Parser, xpath::Context};

use crate::utils::Xpath;

pub struct DefaultBookmarkExtractor {
    pub options: BookmarkExtractorOptions,
}

impl Extractor<ExtractedBookmark> for DefaultBookmarkExtractor {
    fn extract(&self, url: &str, raw: &str) -> Result<ExtractedBookmark, ExtractError> {
        let document = Parser::default()
            .parse_string(raw)
            .map_err(|e| ExtractError(e.into()))?;

        let mut context = Context::new(&document)
            .map_err(|_| ExtractError(anyhow!("couldn't create xpath context from document")))?;

        let bookmark = ExtractedBookmark {
            link: context
                .find_first_content(self.options.link_expr, None)
                .or(Some(url.to_owned())),
            title: context.find_first_content(self.options.title_expr, None),
            thumbnail: context.find_first_content(self.options.thumbnail_expr, None),
            published: context.find_first_content(self.options.published_expr, None),
            author: context.find_first_content(self.options.author_expr, None),
        };

        Ok(bookmark)
    }
}
