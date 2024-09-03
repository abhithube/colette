use colette_scraper::{downloader::DownloaderPlugin, feed::FeedPlugin, DownloaderError};
use http::Request;

const DOWNLOADER_PLUGIN: DownloaderPlugin = DownloaderPlugin::Callback(|url| {
    if !url.path().contains(".rss") {
        url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
    }

    Request::get(url.as_ref())
        .body(())
        .map(|e| e.into_parts().0)
        .map_err(|e| DownloaderError(e.into()))
});

pub fn new_reddit_feed_plugin() -> FeedPlugin<'static> {
    FeedPlugin {
        downloader: Some(DOWNLOADER_PLUGIN),
        ..Default::default()
    }
}
