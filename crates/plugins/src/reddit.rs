use colette_scraper::{
    utils::{ExtractorQuery, Node},
    BookmarkExtractor, BookmarkExtractorOptions, BookmarkScraper, DownloaderError,
    ProcessedBookmark,
};
use http::{header, Method};
use reqwest::Client;
use scraper::Selector;
use url::Url;

#[derive(Clone)]
pub struct RedditBookmarkPlugin<'a> {
    client: Client,
    extractor: BookmarkExtractor<'a>,
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
impl<'a> BookmarkScraper for RedditBookmarkPlugin<'a> {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedBookmark, colette_scraper::Error> {
        if !url.path().contains(".rss") {
            url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
        }

        let resp = self.client
            .request(Method::GET, url.as_str())
            .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
            .send()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        let body = resp
            .bytes()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        let bookmark = self.extractor.extract(body)?;

        Ok(bookmark.try_into()?)
    }
}
