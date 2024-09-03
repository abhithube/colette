use colette_scraper::{feed::FeedPlugin, DownloaderError};
use http::Request;

pub fn new_reddit_feed_plugin() -> FeedPlugin<'static> {
    FeedPlugin {
        downloader: Some(|url| {
            if !url.path().contains(".rss") {
                url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
            }

            Request::get(url.as_ref())
                .body(())
                .map(|e| e.into_parts().0)
                .map_err(|e| DownloaderError(e.into()))
        }),
        ..Default::default()
    }
}
