use colette_http::Client;
use colette_scraper::{
    bookmark::{BookmarkExtractor, BookmarkExtractorOptions, BookmarkScraper, ProcessedBookmark},
    utils::{ExtractorQuery, Node},
    DownloaderError,
};
use http::{header, HeaderMap, HeaderValue};
use scraper::Selector;
use url::Url;

#[derive(Clone)]
pub struct RedditBookmarkPlugin {
    client: Client,
    extractor: BookmarkExtractor,
}

pub fn bookmark(client: Client) -> Box<dyn BookmarkScraper> {
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

    Box::new(RedditBookmarkPlugin {
        client,
        extractor: BookmarkExtractor::new(options),
    })
}

#[async_trait::async_trait]
impl BookmarkScraper for RedditBookmarkPlugin {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, colette_scraper::Error> {
        if !url.path().contains(".rss") {
            url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
        }

        let mut headers = HeaderMap::new();
        headers.insert(header::USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36"));

        let body = self
            .client
            .get(url.as_str(), Some(headers))
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        let bookmark = self.extractor.extract(body)?;

        Ok(bookmark.try_into()?)
    }
}
