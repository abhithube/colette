use colette_core::utils::scraper::{DownloadError, DownloaderPlugin};
use http::Request;
use regex::Regex;
use url::Url;

pub const DOWNLOADER_PLUGIN: DownloaderPlugin = DownloaderPlugin::Callback(|url| {
    let channel_regex =
        Regex::new(r#"/channel/(UC[\w_-]+)"#).expect("failed to create channel regex");

    let mut parsed = Url::parse(url).map_err(|e| DownloadError(e.into()))?;
    if let Some(captures) = channel_regex.captures(url) {
        if let Some(m) = captures.get(1) {
            parsed.set_path("feeds/videos.xml");
            parsed.set_query(Some(&format!("channel_id={}", m.as_str())));
        }
    }

    Request::get(parsed.as_ref())
        .body(())
        .map_err(|e| DownloadError(e.into()))
});
