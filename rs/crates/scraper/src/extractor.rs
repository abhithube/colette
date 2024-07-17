use anyhow::anyhow;
use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed, ExtractorOptions},
    utils::scraper::{ExtractError, Extractor},
};
use libxml::{parser::Parser, tree::Node, xpath::Context};

pub struct DefaultFeedExtractor {
    pub options: ExtractorOptions,
}

trait Xpath {
    fn find_first_content(&mut self, exprs: &'static [&str], node: Option<&Node>)
        -> Option<String>;

    fn find_nodes(&mut self, exprs: &'static [&str], node: Option<&Node>) -> Vec<Node>;
}

impl Xpath for Context {
    fn find_first_content(
        &mut self,
        exprs: &'static [&str],
        node: Option<&Node>,
    ) -> Option<String> {
        exprs
            .iter()
            .find_map(|expr| self.findvalue(expr, node).ok())
    }

    fn find_nodes(&mut self, exprs: &'static [&str], node: Option<&Node>) -> Vec<Node> {
        exprs
            .iter()
            .find_map(|expr| self.findnodes(expr, node).ok())
            .unwrap_or(vec![])
    }
}

impl Extractor<ExtractedFeed> for DefaultFeedExtractor {
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
            .find_nodes(self.options.feed_entries_expr, None)
            .iter()
            .map(|node| ExtractedEntry {
                link: context.find_first_content(self.options.entry_link_expr, Some(node)),
                title: context.find_first_content(self.options.entry_title_expr, Some(node)),
                published: context
                    .find_first_content(self.options.entry_published_expr, Some(node)),
                description: context
                    .find_first_content(self.options.entry_description_expr, Some(node)),
                author: context.find_first_content(self.options.entry_author_expr, Some(node)),
                thumbnail: context
                    .find_first_content(self.options.entry_thumbnail_expr, Some(node)),
            })
            .collect();

        let feed = ExtractedFeed {
            link: context
                .find_first_content(self.options.feed_link_expr, None)
                .or(Some(url.to_owned())),
            title: context.find_first_content(self.options.feed_title_expr, None),
            entries,
        };

        Ok(feed)
    }
}
