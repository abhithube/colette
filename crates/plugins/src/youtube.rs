use bytes::Bytes;
use colette_scraper::{Downloader, DownloaderError, FeedScraper};
use lazy_regex::regex_captures;
use reqwest::Client;
use url::Url;

#[derive(Clone)]
pub struct YouTubePlugin {
    client: Client,
}

pub fn create(client: Client) -> Box<dyn FeedScraper> {
    Box::new(YouTubePlugin { client })
}

#[async_trait::async_trait]
impl Downloader for YouTubePlugin {
    async fn download(&self, url: &mut Url) -> Result<Bytes, DownloaderError> {
        if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.as_str()) {
            url.set_query(Some(&format!("channel_id={}", channel_id)));
            url.set_path("feeds/videos.xml");
        }

        let resp = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))?;

        resp.bytes()
            .await
            .map_err(|e: reqwest::Error| DownloaderError(e.into()))
    }
}

impl FeedScraper for YouTubePlugin {}
