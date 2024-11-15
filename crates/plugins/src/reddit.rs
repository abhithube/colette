use colette_scraper::{
    utils::{ExtractorQuery, Node},
    BookmarkExtractorOptions, BookmarkScraper, Downloader, FeedScraper,
};
use http::{header, request::Builder, Request};
use scraper::Selector;
use url::Url;

#[derive(Clone)]
pub struct RedditPlugin;

pub fn feed() -> Box<dyn FeedScraper> {
    Box::new(RedditPlugin)
}

impl Downloader for RedditPlugin {
    fn before_download(&self, url: &mut Url) -> Builder {
        if !url.path().contains(".rss") {
            url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
        }

        Request::get(url.as_ref())
            .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
    }
}

impl FeedScraper for RedditPlugin {}

pub fn bookmark() -> Box<dyn BookmarkScraper> {
    Box::new(RedditPlugin)
}

impl BookmarkScraper for RedditPlugin {
    fn before_extract(&self) -> Option<BookmarkExtractorOptions> {
        Some(BookmarkExtractorOptions {
            title_queries: vec![ExtractorQuery {
                selector: Selector::parse("shreddit-post").unwrap(),
                node: Node::Attr("post-title"),
            }],
            thumbnail_queries: vec![ExtractorQuery {
                selector: Selector::parse(".preview-img").unwrap(),
                node: Node::Attr("src"),
            }],
            published_queries: vec![ExtractorQuery {
                selector: Selector::parse("shreddit-post").unwrap(),
                node: Node::Attr("created-timestamp"),
            }],
            author_queries: vec![ExtractorQuery {
                selector: Selector::parse("shreddit-post").unwrap(),
                node: Node::Attr("author"),
            }],
        })
    }
}
