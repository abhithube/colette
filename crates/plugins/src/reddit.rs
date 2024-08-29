use colette_scraper::downloader::{DownloaderPlugin, Error};
use http::Request;
use url::Url;

pub const DOWNLOADER_PLUGIN: DownloaderPlugin = DownloaderPlugin::Callback(|url| {
    let mut parsed = Url::parse(url).map_err(|e| Error(e.into()))?;

    if !parsed.path().contains(".rss") {
        parsed
            .path_segments_mut()
            .unwrap()
            .pop_if_empty()
            .push(".rss");
    }

    Request::get(parsed.as_ref())
        .body(())
        .map_err(|e| Error(e.into()))
});
