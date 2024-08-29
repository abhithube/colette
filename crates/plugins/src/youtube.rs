use colette_scraper::downloader::{DownloaderPlugin, Error};
use http::Request;
use lazy_regex::regex_captures;

pub const DOWNLOADER_PLUGIN: DownloaderPlugin = DownloaderPlugin::Callback(|url| {
    if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.clone().as_str())
    {
        url.set_path("feeds/videos.xml");
        url.set_query(Some(&format!("channel_id={}", channel_id)));
    }

    Request::get(url.as_ref())
        .body(())
        .map(|e| e.into_parts().0)
        .map_err(|e| Error(e.into()))
});
