use std::sync::Arc;

use colette_core::utils::scraper::{DownloadError, Downloader};
use http::Response;
use regex::Regex;
use url::Url;

pub struct YouTubeFeedPlugin {
    channel_regex: Regex,
    downloader: Arc<dyn Downloader>,
}

impl YouTubeFeedPlugin {
    pub fn new(downloader: Arc<dyn Downloader>) -> Self {
        Self {
            channel_regex: Regex::new(r#"/channel/(UC[\w_-]+)"#)
                .expect("failed to create channel regex"),
            downloader,
        }
    }
}

impl Downloader for YouTubeFeedPlugin {
    fn download(&self, url: &mut String) -> Result<Response<String>, DownloadError> {
        if let Some(captures) = self.channel_regex.captures(url) {
            let mut parsed = Url::parse(url).map_err(|e| DownloadError(e.into()))?;
            if let Some(m) = captures.get(1) {
                parsed.set_path("feeds/videos.xml");
                parsed.set_query(Some(&format!("channel_id={}", m.as_str())));
            }

            *url = parsed.to_string();
        }

        self.downloader.download(url)
    }
}
