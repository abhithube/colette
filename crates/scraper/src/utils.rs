use colette_core::utils::scraper::{ExtractorQuery, Node};
use scraper::{ElementRef, Html, Selector};

pub trait TextSelector {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String>;
}

impl TextSelector for Html {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&Selector::parse(item.selector).unwrap())
                .next()
                .and_then(|e| select(e, &item.node))
        })
    }
}

impl TextSelector for ElementRef<'_> {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&Selector::parse(item.selector).unwrap())
                .next()
                .and_then(|e| select(e, &item.node))
        })
    }
}

pub fn select(e: ElementRef, node: &Node<'_>) -> Option<String> {
    match node {
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
