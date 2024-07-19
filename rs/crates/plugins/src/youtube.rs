use std::sync::Arc;

use bytes::Bytes;
use colette_core::utils::scraper::{DownloadError, Downloader};
use http::Response;
use regex::Regex;
use reqwest::Url;

pub struct YouTubeFeedPlugin {
    channel_regex: Regex,
    downloader: Arc<dyn Downloader + Send + Sync>,
}

impl YouTubeFeedPlugin {
    pub fn new(downloader: Arc<dyn Downloader + Send + Sync>) -> Self {
        Self {
            channel_regex: Regex::new(r#"/channel/(UC[\w_-]+)"#)
                .expect("failed to create channel regex"),
            downloader,
        }
    }
}

#[async_trait::async_trait]
impl Downloader for YouTubeFeedPlugin {
    async fn download(&self, url: &str) -> Result<Response<Bytes>, DownloadError> {
        let mut parsed = Url::parse(url).map_err(|e| DownloadError(e.into()))?;

        if let Some(captures) = self.channel_regex.captures(url) {
            if let Some(m) = captures.get(1) {
                parsed.set_path("feeds/videos.xml");
                parsed.set_query(Some(&format!("channel_id={}", m.as_str())));
            }
        }

        self.downloader.download(parsed.as_str()).await
    }
}
