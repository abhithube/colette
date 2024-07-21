use colette_core::utils::scraper::{DownloadError, DownloaderPlugin};
use http::Request;
use lazy_regex::regex_captures;
use url::Url;

pub const DOWNLOADER_PLUGIN: DownloaderPlugin = DownloaderPlugin::Callback(|url| {
    let mut parsed = Url::parse(url).map_err(|e| DownloadError(e.into()))?;

    if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url) {
        parsed.set_path("feeds/videos.xml");
        parsed.set_query(Some(&format!("channel_id={}", channel_id)));
    }

    Request::get(parsed.as_ref())
        .body(())
        .map_err(|e| DownloadError(e.into()))
});
