use colette_http::{HttpClient, HyperClient};
use colette_scraper::{
    bookmark::{BookmarkExtractor, BookmarkExtractorOptions, BookmarkScraper, ProcessedBookmark},
    utils::{ExtractorQuery, Node},
};
use http::{HeaderValue, Request, header};
use http_body_util::BodyExt;
use scraper::Selector;
use url::Url;

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct RedditBookmarkPlugin {
    client: HyperClient,
    extractor: BookmarkExtractor,
}

impl RedditBookmarkPlugin {
    pub fn new(client: HyperClient) -> Self {
        let options = BookmarkExtractorOptions {
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
        };

        Self {
            client,
            extractor: BookmarkExtractor::new(options),
        }
    }
}

#[async_trait::async_trait]
impl BookmarkScraper for RedditBookmarkPlugin {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, colette_scraper::Error> {
        if !url.path().contains(".rss") {
            url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
        }

        let req = Request::get(url.as_str())
            .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
            .body(Default::default())
            .unwrap();

        let resp = self.client.send(req).await?;
        let body = resp
            .collect()
            .await
            .map_err(colette_http::Error::Http)?
            .to_bytes();

        let bookmark = self.extractor.extract(body)?;

        Ok(bookmark.try_into()?)
    }
}
