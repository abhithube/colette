use std::sync::Arc;

use colette_core::utils::scraper::{DownloadError, Downloader};
use http::Response;
use url::Url;

pub struct RedditFeedPlugin {
    downloader: Arc<dyn Downloader>,
}

impl RedditFeedPlugin {
    pub fn new(downloader: Arc<dyn Downloader>) -> Self {
        Self { downloader }
    }
}

impl Downloader for RedditFeedPlugin {
    fn download(&self, url: &mut String) -> Result<Response<String>, DownloadError> {
        let mut parsed = Url::parse(url).map_err(|e| DownloadError(e.into()))?;

        if !parsed.path().contains(".rss") {
            parsed
                .path_segments_mut()
                .unwrap()
                .pop_if_empty()
                .push(".rss");
        }

        *url = parsed.to_string();

        self.downloader.download(url)
    }
}
