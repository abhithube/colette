use scraper::{ElementRef, Html, Selector};

#[derive(Clone, Debug)]
pub enum Node<'a> {
    Text,
    Attr(&'a str),
}

#[derive(Clone, Debug)]
pub struct ExtractorQuery<'a> {
    pub selector: Selector,
    pub node: Node<'a>,
}

impl<'a> ExtractorQuery<'a> {
    pub fn new(selector: Selector, node: Node<'a>) -> Self {
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
