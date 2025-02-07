use scraper::{ElementRef, Html, Selector};

#[derive(Debug, Clone)]
pub enum Node {
    Text,
    Attr(&'static str),
}

#[derive(Debug, Clone)]
pub struct ExtractorQuery {
    pub selector: Selector,
    pub node: Node,
}

impl ExtractorQuery {
    pub fn new(selector: Selector, node: Node) -> Self {
        Self { selector, node }
    }
}

pub trait TextSelector {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String>;
}

impl TextSelector for Html {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&item.selector)
                .next()
                .and_then(|e| select(e, &item.node))
        })
    }
}

impl TextSelector for ElementRef<'_> {
    fn select_text(&self, items: &[ExtractorQuery]) -> Option<String> {
        items.iter().find_map(|item| {
            self.select(&item.selector)
                .next()
                .and_then(|e| select(e, &item.node))
        })
    }
}

pub fn select(e: ElementRef, node: &Node) -> Option<String> {
    match node {
        Node::Text => {
            let text = e.inner_html();
            (!text.is_empty()).then_some(text)
        }
        Node::Attr(attr) => e
            .attr(attr)
            .and_then(|e| (!e.is_empty()).then_some(e.to_owned())),
    }
}
