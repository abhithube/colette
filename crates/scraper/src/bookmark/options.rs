use super::BookmarkExtractorOptions;
use crate::{ExtractorQuery, Node};

pub fn base_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_queries: vec![
            ExtractorQuery::new("meta[name='title']", Node::Attr("content")),
            ExtractorQuery::new("title", Node::Text),
        ],
        published_queries: vec![],
        author_queries: vec![],
        thumbnail_queries: vec![],
    }
}

pub fn open_graph_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_queries: vec![ExtractorQuery::new(
            "meta[property='og:title']",
            Node::Attr("content"),
        )],
        published_queries: vec![],
        author_queries: vec![],
        thumbnail_queries: vec![ExtractorQuery::new(
            "meta[property='og:image']",
            Node::Attr("content"),
        )],
    }
}

pub fn twitter_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_queries: vec![ExtractorQuery::new(
            "meta[name='twitter:title']",
            Node::Attr("content"),
        )],
        published_queries: vec![],
        author_queries: vec![],
        thumbnail_queries: vec![ExtractorQuery::new(
            "meta[name='twitter:image']",
            Node::Attr("content"),
        )],
    }
}

pub fn microdata_extractor_options<'a>() -> BookmarkExtractorOptions<'a> {
    BookmarkExtractorOptions {
        title_queries: vec![ExtractorQuery::new(
            "[itemtype='http://schema.org/VideoObject'] > [itemprop='name']",
            Node::Attr("content"),
        )],
        published_queries: vec![
            ExtractorQuery::new(
                "[itemtype='http://schema.org/VideoObject'] > [itemprop='datePublished']",
                Node::Attr("content"),
            ),
            ExtractorQuery::new(
                "[itemtype='http://schema.org/VideoObject'] > [itemprop='uploadDate']",
                Node::Attr("content"),
            ),
        ],
        author_queries: vec![ExtractorQuery::new(
            "[itemtype='http://schema.org/Person'] > [itemprop='name']",
            Node::Attr("content"),
        )],
        thumbnail_queries: vec![
            ExtractorQuery::new(
                "[itemtype='http://schema.org/ImageObject'] > [itemprop='url']",
                Node::Attr("href"),
            ),
            ExtractorQuery::new("[itemprop='thumbnailUrl']", Node::Attr("href")),
        ],
    }
}
