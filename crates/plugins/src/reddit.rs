use colette_scraper::downloader::{DownloaderPlugin, Error};
use http::Request;

pub const DOWNLOADER_PLUGIN: DownloaderPlugin = DownloaderPlugin::Callback(|url| {
    if !url.path().contains(".rss") {
        url.path_segments_mut().unwrap().pop_if_empty().push(".rss");
    }

    Request::get(url.as_ref())
        .body(())
        .map(|e| e.into_parts().0)
        .map_err(|e| Error(e.into()))
});
