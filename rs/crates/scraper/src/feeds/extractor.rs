use anyhow::anyhow;
use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed, FeedExtractorOptions},
    utils::scraper::{ExtractError, Extractor, ExtractorQuery, Node},
};
use http::Response;
use scraper::{ElementRef, Html, Selector};

use super::{atom::AtomFeed, rss::RSSFeed};

pub struct DefaultFeedExtractor {}

impl Extractor<ExtractedFeed> for DefaultFeedExtractor {
    fn extract(&self, _url: &str, resp: Response<String>) -> Result<ExtractedFeed, ExtractError> {
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

impl Extractor<ExtractedFeed> for HtmlExtractor<'_> {
    fn extract(&self, _url: &str, resp: Response<String>) -> Result<ExtractedFeed, ExtractError> {
        let raw = resp.into_body();
        let html = Html::parse_document(&raw);

        let entries: Vec<ExtractedEntry> = html
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

pub trait TextSelector {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String>;
}

impl TextSelector for Html {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&Selector::parse(item.selector).unwrap())
                .next()
                .and_then(|e| select(e, item))
        })
    }
}

impl TextSelector for ElementRef<'_> {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&Selector::parse(item.selector).unwrap())
                .next()
                .and_then(|e| select(e, item))
        })
    }
}

fn select(e: ElementRef, item: &ExtractorQuery) -> Option<String> {
    match item.node {
        Node::Text => {
            let text = e.inner_html();
            match text.is_empty() {
                true => None,
                false => Some(text),
            }
        }
        Node::Attr(attr) => e.attr(attr).and_then(|e| match e.is_empty() {
            true => None,
            false => Some(e.to_owned()),
        }),
    }
}
