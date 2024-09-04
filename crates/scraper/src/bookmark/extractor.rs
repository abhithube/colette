use scraper::Selector;

use crate::utils::{ExtractorQuery, Node};

#[derive(Clone, Debug)]
pub struct BookmarkExtractorOptions<'a> {
    pub title_queries: Vec<ExtractorQuery<'a>>,
    pub published_queries: Vec<ExtractorQuery<'a>>,
    pub author_queries: Vec<ExtractorQuery<'a>>,
    pub thumbnail_queries: Vec<ExtractorQuery<'a>>,
}

impl<'a> Default for BookmarkExtractorOptions<'a> {
    fn default() -> Self {
        Self {
            title_queries: vec![
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/VideoObject'] > [itemprop='name']",
                    )
                    .unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[property='og:title']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[name='twitter:title']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[name='title']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(Selector::parse("title").unwrap(), Node::Text),
            ],
            published_queries: vec![
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/VideoObject'] > [itemprop='datePublished']",
                    )
                    .unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/VideoObject'] > [itemprop='uploadDate']",
                    )
                    .unwrap(),
                    Node::Attr("content"),
                ),
            ],
            author_queries: vec![ExtractorQuery::new(
                Selector::parse("[itemtype='http://schema.org/Person'] > [itemprop='name']")
                    .unwrap(),
                Node::Attr("content"),
            )],
            thumbnail_queries: vec![
                ExtractorQuery::new(
                    Selector::parse(
                        "[itemtype='http://schema.org/ImageObject'] > [itemprop='url']",
                    )
                    .unwrap(),
                    Node::Attr("href"),
                ),
                ExtractorQuery::new(
                    Selector::parse("[itemprop='thumbnailUrl']").unwrap(),
                    Node::Attr("href"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[property='og:image']").unwrap(),
                    Node::Attr("content"),
                ),
                ExtractorQuery::new(
                    Selector::parse("meta[name='twitter:image']").unwrap(),
                    Node::Attr("content"),
                ),
            ],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ExtractedBookmark {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub published: Option<String>,
    pub author: Option<String>,
}
