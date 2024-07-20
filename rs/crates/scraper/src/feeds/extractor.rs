use colette_core::{
    feeds::{ExtractedEntry, ExtractedFeed},
    utils::scraper::{ExtractError, Extractor},
};
use scraper::{ElementRef, Html, Selector};

#[derive(Clone)]
pub enum Node<'a> {
    Text,
    Attr(&'a str),
}

#[derive(Clone)]
pub struct Item<'a> {
    selector: &'a str,
    node_type: Node<'a>,
}

impl<'a> Item<'a> {
    pub fn new(selector: &'a str, node_type: Node<'a>) -> Self {
        Self {
            selector,
            node_type,
        }
    }
}

struct FeedExtractorOptions<'a> {
    pub feed_link_selectors: Vec<Item<'a>>,
    pub feed_title_selectors: Vec<Item<'a>>,
    pub feed_entries_selector: Item<'a>,
    pub entry_link_selectors: Vec<Item<'a>>,
    pub entry_title_selectors: Vec<Item<'a>>,
    pub entry_published_selectors: Vec<Item<'a>>,
    pub entry_description_selectors: Vec<Item<'a>>,
    pub entry_author_selectors: Vec<Item<'a>>,
    pub entry_thumbnail_selectors: Vec<Item<'a>>,
}

pub struct HtmlExtractor<'a> {
    options: FeedExtractorOptions<'a>,
}

impl Extractor<ExtractedFeed> for HtmlExtractor<'_> {
    fn extract(&self, _url: &str, raw: &str) -> Result<ExtractedFeed, ExtractError> {
        let html = Html::parse_document(raw);

        let entries: Vec<ExtractedEntry> = html
            .select(&Selector::parse(self.options.feed_entries_selector.selector).unwrap())
            .map(|element| ExtractedEntry {
                link: element.select_text(&self.options.entry_link_selectors),
                title: element.select_text(&self.options.entry_title_selectors),
                published: element.select_text(&self.options.entry_published_selectors),
                description: element.select_text(&self.options.entry_description_selectors),
                author: element.select_text(&self.options.entry_author_selectors),
                thumbnail: element.select_text(&self.options.entry_thumbnail_selectors),
            })
            .collect();

        let feed = ExtractedFeed {
            link: html.select_text(&self.options.feed_link_selectors),
            title: html.select_text(&self.options.feed_title_selectors),
            entries,
        };

        Ok(feed)
    }
}

pub trait TextSelector {
    fn select_text(&self, items: &[Item]) -> Option<String>;
}

impl TextSelector for Html {
    fn select_text(&self, items: &[Item]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&Selector::parse(item.selector).unwrap())
                .next()
                .and_then(|e| select(e, item))
        })
    }
}

impl TextSelector for ElementRef<'_> {
    fn select_text(&self, items: &[Item]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&Selector::parse(item.selector).unwrap())
                .next()
                .and_then(|e| select(e, item))
        })
    }
}

fn select(e: ElementRef, item: &Item) -> Option<String> {
    match item.node_type {
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
