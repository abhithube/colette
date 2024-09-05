use colette_scraper::{feed::FeedPlugin, DownloaderError};
use http::{header, Request};

pub fn new_reddit_feed_plugin() -> FeedPlugin<'static> {
    FeedPlugin {
        downloader: |url| {
            if !url.path().contains(".rss") {
                url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
            }

            Request::get(url.as_ref())
                .header(header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
                .body(())
                .map(|e| e.into_parts().0)
                .map_err(|e| DownloaderError(e.into()))
        },
        ..Default::default()
    }
}
