use core::str;
use std::io::BufReader;

use bytes::Buf;
use colette_http::HttpClient;
use colette_scraper::feed::FeedScraper;
use futures::stream::BoxStream;
use url::Url;

use super::{Error, Feed, FeedRepository};

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
        let body = self.client.get(&data.url).await?;

        let mut reader = BufReader::new(body.reader());

        let raw = str::from_utf8(reader.peek(14)?)?;
        match raw {
            raw if raw.contains("<!DOCTYPE html") => {
                let metadata = colette_meta::parse_metadata(reader)
                    .map_err(|_| colette_scraper::feed::FeedError::Unsupported)?;

                let mut detected = metadata
                    .feeds
                    .into_iter()
                    .map(FeedDetected::from)
                    .collect::<Vec<_>>();
                if detected.is_empty() {
                    let processed = self.scraper.scrape(&mut data.url).await?;

                    detected.push(FeedDetected {
                        url: data.url,
                        title: processed.title,
                    });
                }

                Ok(detected)
            }
            raw if raw.contains("<?xml") => {
                let processed = self.scraper.scrape(&mut data.url).await?;

                Ok(vec![FeedDetected {
                    url: data.url,
                    title: processed.title,
                }])
            }
            _ => Err(Error::Scraper(
                colette_scraper::feed::FeedError::Unsupported,
            )),
        }
    }

    pub async fn refresh_feed(&self, mut data: FeedRefresh) -> Result<Feed, Error> {
        let processed = self.scraper.scrape(&mut data.url).await?;

        let is_custom = processed.link == data.url;

        let mut feed: Feed = (data.url, processed).into();
        feed.is_custom = is_custom;

        self.repository.save(&feed).await?;

        Ok(feed)
    }

    pub async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        self.repository.stream().await
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
