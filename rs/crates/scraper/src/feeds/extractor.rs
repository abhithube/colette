use anyhow::anyhow;
use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions},
    utils::scraper::{ExtractError, Extractor},
};
use itertools::Itertools;
use libxml::{parser::Parser, tree::Namespace, xpath::Context};

use crate::{
    atom_extractor_options, dublin_core_extractor_options, itunes_extractor_options,
    media_extractor_options, rss_extractor_options, utils::Xpath,
};

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

        let mut options_vec = vec![self.options.clone()];

        if raw.contains("<feed") {
            context
                .register_namespace("atom", "http://www.w3.org/2005/Atom")
                .map_err(|_| ExtractError(anyhow!("couldn't register namespace")))?;

            options_vec.push(atom_extractor_options())
        }
        if raw.contains("<rss") {
            options_vec.push(rss_extractor_options())
        }

        let namespaces = context
            .findnodes("/*[1]", None)
            .ok()
            .and_then(|e| {
                e.first().map(|e| {
                    e.get_namespace_declarations()
                        .into_iter()
                        .filter(|e| !e.get_prefix().is_empty())
                        .collect::<Vec<Namespace>>()
                })
            })
            .unwrap_or(Vec::new());

        for namespace in namespaces {
            let (prefix, href) = (namespace.get_prefix(), namespace.get_href());

            context
                .register_namespace(&prefix, &href)
                .map_err(|_| ExtractError(anyhow!("couldn't register namespace")))?;

            if href == "http://search.yahoo.com/mrss/" {
                options_vec.push(media_extractor_options())
            } else if href == "http://purl.org/dc/elements/1.1/" {
                options_vec.push(dublin_core_extractor_options())
            } else if href == "http://www.itunes.com/dtds/podcast-1.0.dtd" {
                options_vec.push(itunes_extractor_options())
            }
        }

        let options = merge(options_vec.into_iter().unique().collect());

        let entries: Vec<ExtractedEntry> = context
            .find_nodes(&options.feed_entries_expr, None)
            .iter()
            .map(|node| ExtractedEntry {
                link: context.find_first_content(&options.entry_link_expr, Some(node)),
                title: context.find_first_content(&options.entry_title_expr, Some(node)),
                published: context.find_first_content(&options.entry_published_expr, Some(node)),
                description: context
                    .find_first_content(&options.entry_description_expr, Some(node)),
                author: context.find_first_content(&options.entry_author_expr, Some(node)),
                thumbnail: context.find_first_content(&options.entry_thumbnail_expr, Some(node)),
            })
            .collect();

        let feed = ExtractedFeed {
            link: context
                .find_first_content(&options.feed_link_expr, None)
                .or(Some(url.to_owned())),
            title: context.find_first_content(&options.feed_title_expr, None),
            entries,
        };

        Ok(feed)
    }
}

fn merge(options_vec: Vec<FeedExtractorOptions>) -> FeedExtractorOptions {
    fn merge_field<'a>(fields: &[Vec<&'a str>]) -> Vec<&'a str> {
        fields.iter().flat_map(|v| v.iter().cloned()).collect()
    }

    FeedExtractorOptions {
        feed_link_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.feed_link_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        feed_title_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.feed_title_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        feed_entries_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.feed_entries_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        entry_link_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.entry_link_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        entry_title_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.entry_title_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        entry_published_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.entry_published_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        entry_description_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.entry_description_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        entry_author_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.entry_author_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
        entry_thumbnail_expr: merge_field(
            &options_vec
                .iter()
                .map(|e| e.entry_thumbnail_expr.clone())
                .filter(|e| !e.is_empty())
                .collect::<Vec<_>>(),
        ),
    }
}
