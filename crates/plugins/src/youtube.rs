use std::io::BufReader;

use bytes::Buf;
use colette_scraper::feed::{ExtractedFeed, FeedError, FeedPlugin, ProcessedFeed};
use lazy_regex::regex_captures;
use reqwest::{Client, Url};

#[derive(Clone)]
pub struct YouTubeFeedPlugin {
    client: Client,
}

impl YouTubeFeedPlugin {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl FeedPlugin for YouTubeFeedPlugin {
    async fn scrape(&self, url: &mut Url) -> Result<ProcessedFeed, FeedError> {
        if let Some((_, channel_id)) = regex_captures!(r#"/channel/(UC[\w_-]+)"#, url.as_str()) {
            url.set_query(Some(&format!("channel_id={channel_id}")));
            url.set_path("feeds/videos.xml");
        }

        let resp = self.client.get(url.to_owned()).send().await?;
        let body = resp.bytes().await?;

        let extracted = colette_feed::from_reader(BufReader::new(body.reader()))
            .map(ExtractedFeed::from)
            .map_err(FeedError::Parse)?;

        Ok(extracted.try_into()?)
    }
}
