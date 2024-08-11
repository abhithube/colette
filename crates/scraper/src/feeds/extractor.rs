use anyhow::anyhow;
use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions},
    scraper::{ExtractError, Extractor},
};
use http::Response;
use scraper::{Html, Selector};
use url::Url;

use super::{atom::AtomFeed, rss::RSSFeed};
use crate::utils::TextSelector;

pub struct DefaultFeedExtractor {}

impl Extractor for DefaultFeedExtractor {
    type T = ExtractedFeed;

    fn extract(&self, _url: &Url, resp: Response<String>) -> Result<ExtractedFeed, ExtractError> {
        let (parts, body) = resp.into_parts();

        let content_type = parts
            .headers
            .get(http::header::CONTENT_TYPE)
            .and_then(|e| e.to_str().ok());

        let feed = if content_type.map_or(false, |e| e.contains("application/atom+xml"))
            || body.contains("<feed")
        {
            quick_xml::de::from_str::<AtomFeed>(&body)
                .map(ExtractedFeed::from)
                .map_err(|e| ExtractError(e.into()))
        } else if content_type.map_or(false, |e| e.contains("application/rss+xml"))
            || body.contains("<rss")
        {
            quick_xml::de::from_str::<RSSFeed>(&body)
                .map(ExtractedFeed::from)
                .map_err(|e| ExtractError(e.into()))
        } else {
            Err(ExtractError(anyhow!(
                "couldn't find extractor for feed URL"
            )))
        }?;

        Ok(feed)
    }
}

pub struct HtmlExtractor<'a> {
    options: FeedExtractorOptions<'a>,
}

impl Extractor for HtmlExtractor<'_> {
    type T = ExtractedFeed;

    fn extract(&self, _url: &Url, resp: Response<String>) -> Result<ExtractedFeed, ExtractError> {
        let raw = resp.into_body();
        let html = Html::parse_document(&raw);

        let entries = html
            .select(&Selector::parse(self.options.feed_entries_selector).unwrap())
            .map(|element| ExtractedEntry {
                link: element.select_text(&self.options.entry_link_queries),
                title: element.select_text(&self.options.entry_title_queries),
                published: element.select_text(&self.options.entry_published_queries),
                description: element.select_text(&self.options.entry_description_queries),
                author: element.select_text(&self.options.entry_author_queries),
                thumbnail: element.select_text(&self.options.entry_thumbnail_queries),
            })
            .collect();

        let feed = ExtractedFeed {
            link: html.select_text(&self.options.feed_link_queries),
            title: html.select_text(&self.options.feed_title_queries),
            entries,
        };

        Ok(feed)
    }
}
