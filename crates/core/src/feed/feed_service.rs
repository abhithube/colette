use core::str;

use bytes::Buf;
use colette_http::HttpClient;
use colette_scraper::feed::FeedScraper;
use futures::{
    StreamExt, TryFutureExt,
    stream::{self, BoxStream},
};
use url::Url;

use super::{Error, Feed, FeedParams, FeedRepository};

pub struct FeedService {
    repository: Box<dyn FeedRepository>,
    client: Box<dyn HttpClient>,
    scraper: FeedScraper,
}

impl FeedService {
    pub fn new(
        repository: impl FeedRepository,
        client: impl HttpClient,
        scraper: FeedScraper,
    ) -> Self {
        Self {
            repository: Box::new(repository),
            client: Box::new(client),
            scraper,
        }
    }

    pub async fn detect_feeds(&self, mut data: FeedDetect) -> Result<Vec<FeedDetected>, Error> {
        match self.scraper.scrape(&mut data.url).await {
            Ok(processed) => {
                let detected = vec![FeedDetected {
                    url: data.url,
                    title: processed.title,
                }];

                Ok(detected)
            }
            Err(colette_scraper::feed::FeedError::Unsupported) => {
                let body = self.client.get(&data.url).await?;

                let metadata = colette_meta::parse_metadata(body.reader())
                    .map_err(|_| colette_scraper::feed::FeedError::Unsupported)?;

                let detected = metadata
                    .feeds
                    .into_iter()
                    .map(FeedDetected::from)
                    .collect::<Vec<_>>();

                Ok(detected)
            }
            Err(e) => Err(Error::Scraper(e)),
        }
    }

    pub async fn refresh_feed(&self, mut data: FeedRefresh) -> Result<Feed, Error> {
        let processed = self.scraper.scrape(&mut data.url).await?;

        let is_custom = processed.link == data.url;

        let mut feed: Feed = (data.url, processed).into();
        feed.is_custom = is_custom;

        self.repository.save(&mut feed).await?;

        Ok(feed)
    }

    pub async fn stream(&'_ self) -> Result<BoxStream<'_, Url>, Error> {
        let x = self
            .repository
            .query(FeedParams::default())
            .map_ok(|e| e.into_iter().map(|e| e.source_url).collect::<Vec<_>>())
            .await?;

        Ok(stream::iter(x).boxed())
    }
}

#[derive(Debug, Clone)]
pub struct FeedDetect {
    pub url: Url,
}

#[derive(Debug, Clone)]
pub struct FeedDetected {
    pub url: Url,
    pub title: String,
}

impl From<colette_meta::rss::Feed> for FeedDetected {
    fn from(value: colette_meta::rss::Feed) -> Self {
        Self {
            url: value.href.parse().unwrap(),
            title: value.title,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeedRefresh {
    pub url: Url,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ScrapeFeedJobData {
    pub url: Url,
}
