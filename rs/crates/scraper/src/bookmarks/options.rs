use super::extractor::BookmarkExtractorOptions;
use crate::feeds::{Item, Node};

pub fn base_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_selectors: vec![
            Item::new("meta[name='title']", Node::Attr("content")),
            Item::new("title", Node::Text),
        ],
        published_selectors: vec![],
        author_selectors: vec![],
        thumbnail_selectors: vec![],
    }
}

pub fn open_graph_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_selectors: vec![Item::new(
            "meta[property='og:title']",
            Node::Attr("content"),
        )],
        published_selectors: vec![],
        author_selectors: vec![],
        thumbnail_selectors: vec![Item::new(
            "meta[property='og:image']",
            Node::Attr("content"),
        )],
    }
}

pub fn twitter_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_selectors: vec![Item::new(
            "meta[name='twitter:title']",
            Node::Attr("content"),
        )],
        published_selectors: vec![],
        author_selectors: vec![],
        thumbnail_selectors: vec![Item::new(
            "meta[name='twitter:image']",
            Node::Attr("content"),
        )],
    }
}

pub fn microdata_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_selectors: vec![Item::new(
            "[itemtype='http://schema.org/VideoObject'] > [itemprop='name']",
            Node::Attr("content"),
        )],
        published_selectors: vec![
            Item::new(
                "[itemtype='http://schema.org/VideoObject'] > [itemprop='datePublished']",
                Node::Attr("content"),
            ),
            Item::new(
                "[itemtype='http://schema.org/VideoObject'] > [itemprop='uploadDate']",
                Node::Attr("content"),
            ),
        ],
        author_selectors: vec![Item::new(
            "[itemtype='http://schema.org/Person'] > [itemprop='name']",
            Node::Attr("content"),
        )],
        thumbnail_selectors: vec![
            Item::new(
                "[itemtype='http://schema.org/ImageObject'] > [itemprop='url']",
                Node::Attr("href"),
            ),
            Item::new("[itemprop='thumbnailUrl']", Node::Attr("href")),
        ],
    }
}
