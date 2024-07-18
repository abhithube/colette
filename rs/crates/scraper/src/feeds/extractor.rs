use anyhow::anyhow;
use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions},
    utils::scraper::{ExtractError, Extractor},
};
use libxml::{parser::Parser, xpath::Context};

use crate::utils::Xpath;

pub struct DefaultFeedExtractor<'a> {
    pub options: FeedExtractorOptions<'a>,
}

impl Extractor<ExtractedFeed> for DefaultFeedExtractor<'_> {
    fn extract(&self, url: &str, raw: &str) -> Result<ExtractedFeed, ExtractError> {
        let document = Parser::default()
            .parse_string(raw)
            .map_err(|e| ExtractError(e.into()))?;

        let mut context = Context::new(&document)
            .map_err(|_| ExtractError(anyhow!("couldn't create xpath context from document")))?;

        context
            .register_namespace("atom", "http://www.w3.org/2005/Atom")
            .map_err(|_| ExtractError(anyhow!("couldn't register namespace")))?;

        context
            .register_namespace("media", "http://search.yahoo.com/mrss/")
            .map_err(|_| ExtractError(anyhow!("couldn't register namespace")))?;

        context
            .register_namespace("dc", "http://purl.org/dc/elements/1.1/")
            .map_err(|_| ExtractError(anyhow!("couldn't register namespace")))?;

        let entries: Vec<ExtractedEntry> = context
            .find_nodes(&self.options.feed_entries_expr, None)
            .iter()
            .map(|node| ExtractedEntry {
                link: context.find_first_content(&self.options.entry_link_expr, Some(node)),
                title: context.find_first_content(&self.options.entry_title_expr, Some(node)),
                published: context
                    .find_first_content(&self.options.entry_published_expr, Some(node)),
                description: context
                    .find_first_content(&self.options.entry_description_expr, Some(node)),
                author: context.find_first_content(&self.options.entry_author_expr, Some(node)),
                thumbnail: context
                    .find_first_content(&self.options.entry_thumbnail_expr, Some(node)),
            })
            .collect();

        let feed = ExtractedFeed {
            link: context
                .find_first_content(&self.options.feed_link_expr, None)
                .or(Some(url.to_owned())),
            title: context.find_first_content(&self.options.feed_title_expr, None),
            entries,
        };

        Ok(feed)
    }
}
