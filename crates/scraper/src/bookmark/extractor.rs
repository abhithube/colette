use std::io::BufRead;

use http::Response;
use scraper::{Html, Selector};
use url::Url;

use crate::{
    extractor::{Error, Extractor},
    utils::{ExtractorQuery, Node, TextSelector},
};

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

pub struct DefaultBookmarkExtractor<'a> {
    options: BookmarkExtractorOptions<'a>,
}

impl<'a> DefaultBookmarkExtractor<'a> {
    pub fn new(options: Option<BookmarkExtractorOptions<'a>>) -> Self {
        Self {
            options: options.unwrap_or_default(),
        }
    }
}

impl Extractor for DefaultBookmarkExtractor<'_> {
    type Extracted = ExtractedBookmark;

    fn extract(
        &self,
        _url: &Url,
        resp: Response<Box<dyn BufRead>>,
    ) -> Result<ExtractedBookmark, Error> {
        let mut body = resp.into_body();

        let mut bytes: Vec<u8> = vec![];
        body.read(&mut bytes).map_err(|e| Error(e.into()))?;

        let raw = String::from_utf8_lossy(&bytes);
        let html = Html::parse_document(&raw);

        let bookmark = ExtractedBookmark {
            title: html.select_text(&self.options.title_queries),
            thumbnail: html.select_text(&self.options.thumbnail_queries),
            published: html.select_text(&self.options.published_queries),
            author: html.select_text(&self.options.author_queries),
        };

        Ok(bookmark)
    }
}
