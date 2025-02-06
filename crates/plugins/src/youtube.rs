use std::io::BufReader;

use bytes::Buf;
use colette_core::feed::{ExtractedFeed, FeedScraper, ProcessedFeed, ScraperError};
use colette_http::{HttpClient, HyperClient};
use lazy_regex::regex_captures;
use url::Url;

#[derive(Clone)]
pub struct YouTubeFeedPlugin {
    client: HyperClient,
}

impl YouTubeFeedPlugin {
    pub fn new(client: HyperClient) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl FeedScraper for YouTubeFeedPlugin {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, ScraperError> {
        if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.as_str()) {
            url.set_query(Some(&format!("channel_id={}", channel_id)));
            url.set_path("feeds/videos.xml");
        }

        let (_, body) = self.client.get(url).await?;
        let feed = colette_feed::from_reader(BufReader::new(body.reader()))
            .map(ExtractedFeed::from)
            .map_err(|e| ScraperError::Parse(e.into()))?;

        Ok(feed.try_into()?)
    }
}
