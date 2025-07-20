use core::str;
use std::{cmp, sync::Arc};

use bytes::Buf;
use chrono::{DateTime, Utc};
use colette_http::HttpClient;
use colette_scraper::feed::FeedScraper;
use url::Url;

use super::{Error, Feed, FeedCursor, FeedParams, FeedRepository};
use crate::{
    feed_entry::{FeedEntryParams, FeedEntryRepository},
    pagination::{Paginated, paginate},
};

const DEFAULT_INTERVAL: u64 = 60;
const MIN_INTERVAL: u64 = 5;
const MAX_INTERVAL: u64 = DEFAULT_INTERVAL * 24;
const SAMPLE_SIZE: usize = 20;
const BACKOFF_MULTIPLIER: f64 = 1.15;

pub struct FeedService {
    feed_repository: Arc<dyn FeedRepository>,
    feed_entry_repository: Arc<dyn FeedEntryRepository>,
    client: Box<dyn HttpClient>,
    scraper: FeedScraper,
}

impl FeedService {
    pub fn new(
        feed_repository: Arc<dyn FeedRepository>,
        feed_entry_repository: Arc<dyn FeedEntryRepository>,
        client: impl HttpClient,
        scraper: FeedScraper,
    ) -> Self {
        Self {
            feed_repository,
            feed_entry_repository,
            client: Box::new(client),
            scraper,
        }
    }

    pub async fn list_feeds(
        &self,
        query: FeedListQuery,
    ) -> Result<Paginated<Feed, FeedCursor>, Error> {
        let feeds = self
            .feed_repository
            .query(FeedParams {
                ready_to_refresh: query.ready_to_refresh,
                cursor: query.cursor.map(|e| e.source_url),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(feeds, limit))
        } else {
            Ok(Paginated {
                items: feeds,
                ..Default::default()
            })
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
        let latest_entries = self
            .feed_entry_repository
            .query(FeedEntryParams {
                limit: Some(1),
                ..Default::default()
            })
            .await?;

        let mut processed = self.scraper.scrape(&mut data.url).await?;
        processed
            .entries
            .sort_by(|a, b| a.published.cmp(&b.published));

        let has_new = processed.entries.first().is_some_and(|a| {
            latest_entries
                .first()
                .is_some_and(|b| a.published.gt(&b.published_at))
        });

        let is_custom = processed.link == data.url;

        let mut feed: Feed = (data.url.clone(), processed).into();
        feed.refreshed_at = Some(Utc::now());
        feed.is_custom = is_custom;

        if has_new {
            let mut dates = Vec::<DateTime<Utc>>::new();

            if let Some(entries) = feed.entries.as_ref() {
                for entry in entries.iter().take(SAMPLE_SIZE) {
                    dates.push(entry.published_at);
                }
            }

            if dates.len() < SAMPLE_SIZE {
                let entries = self
                    .feed_entry_repository
                    .query(FeedEntryParams {
                        feed_id: Some(feed.id),
                        limit: Some(SAMPLE_SIZE - dates.len()),
                        ..Default::default()
                    })
                    .await?;

                for entry in entries {
                    dates.push(entry.published_at);
                }
            }

            if dates.len() >= 2 {
                let mut deltas = dates
                    .windows(2)
                    .map(|e| (e[0] - e[1]).num_minutes() as u64)
                    .collect::<Vec<_>>();

                deltas.sort_unstable();

                let median = if deltas.len() % 2 == 0 {
                    let right = deltas.len() / 2;
                    let left = right - 1;
                    (deltas[left] + deltas[right]) / 2
                } else {
                    deltas[deltas.len() / 2]
                };

                feed.refresh_interval_min = median.clamp(MIN_INTERVAL, MAX_INTERVAL);
            }
        } else if let Some(f) = self.feed_repository.find_by_source_url(data.url).await? {
            feed.refresh_interval_min = cmp::min(
                (f.refresh_interval_min as f64 * BACKOFF_MULTIPLIER) as u64,
                MAX_INTERVAL,
            );
        }

        self.feed_repository.save(&mut feed).await?;

        Ok(feed)
    }
}

#[derive(Debug, Clone, Default)]
pub struct FeedListQuery {
    pub ready_to_refresh: bool,
    pub cursor: Option<FeedCursor>,
    pub limit: Option<usize>,
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
