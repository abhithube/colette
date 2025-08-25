use colette_scraper::{
    ExtractorQuery, Node,
    bookmark::{BookmarkError, BookmarkPlugin, ProcessedBookmark},
};
use reqwest::{
    Client, Method, Request, RequestBuilder,
    header::{self, HeaderValue},
};
use scraper::Selector;
use url::Url;

use crate::common::{BookmarkExtractor, BookmarkExtractorOptions};

const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

#[derive(Clone)]
pub struct RedditBookmarkPlugin {
    client: Client,
    extractor: BookmarkExtractor,
}

impl RedditBookmarkPlugin {
    pub fn new(client: Client) -> Self {
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
impl BookmarkPlugin for RedditBookmarkPlugin {
    fn is_supported(&self, url: &mut Url) -> bool {
        if let Some(domain) = url.domain()
            && domain != "www.reddit.com"
        {
            return false;
        }

        if !url.path().contains(".rss") {
            url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
        }

        true
    }

    async fn scrape(&self, url: &Url) -> Result<ProcessedBookmark, BookmarkError> {
        let resp = RequestBuilder::from_parts(
            self.client.clone(),
            Request::new(Method::GET, url.to_owned()),
        )
        .header(header::USER_AGENT, HeaderValue::from_static(USER_AGENT))
        .send()
        .await?;
        let body = resp.bytes().await?;

        let extracted = self.extractor.extract(body)?;

        Ok(extracted.try_into()?)
    }
}
