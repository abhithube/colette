use std::sync::Arc;

use bytes::Bytes;
use colette_core::utils::scraper::{DownloadError, Downloader};
use http::Response;
use reqwest::Url;

pub struct RedditFeedPlugin {
    downloader: Arc<dyn Downloader>,
}

impl RedditFeedPlugin {
    pub fn new(downloader: Arc<dyn Downloader>) -> Self {
        Self { downloader }
    }
}

#[async_trait::async_trait]
impl Downloader for RedditFeedPlugin {
    async fn download(&self, url: &mut String) -> Result<Response<Bytes>, DownloadError> {
        let mut parsed = Url::parse(url).map_err(|e| DownloadError(e.into()))?;

        if !parsed.path().contains(".rss") {
            parsed
                .path_segments_mut()
                .unwrap()
                .pop_if_empty()
                .push(".rss");
        }

        *url = parsed.to_string();

        self.downloader.download(url).await
    }
}
