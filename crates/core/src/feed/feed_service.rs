use core::str;
use std::{cmp, sync::Arc};

use bytes::Buf;
use colette_http::HttpClient;
use colette_scraper::feed::{FeedScraper, ProcessedFeed};
use url::Url;
use uuid::Uuid;

use super::{Error, Feed, FeedCursor, FeedFindParams, FeedRepository};
use crate::{
    feed::FeedUpsertParams,
    feed_entry::{FeedEntryFindParams, FeedEntryRepository},
    pagination::{Paginated, paginate},
};

pub const DEFAULT_INTERVAL: u32 = 60;
const MIN_INTERVAL: u32 = 5;
const MAX_INTERVAL: u32 = DEFAULT_INTERVAL * 24;
const SAMPLE_SIZE: usize = 20;
const BACKOFF_MULTIPLIER: f32 = 1.15;

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
            .find(FeedFindParams {
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

    pub async fn get_feed(&self, id: Uuid) -> Result<Feed, Error> {
        let mut feeds = self
            .feed_repository
            .find(FeedFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(Error::NotFound(id));
        }

        let feed = feeds.swap_remove(0);

        Ok(feed)
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
        let mut processed = match self.scraper.scrape(&mut data.url).await {
            Ok(processed) => Ok(processed),
            Err(e) => {
                self.feed_repository
                    .mark_as_failed(data.url.clone())
                    .await?;

                Err(e)
            }
        }?;

        processed
            .entries
            .sort_by(|a, b| a.published.cmp(&b.published));

        let refresh_interval_min = self
            .calculate_refresh_interval(data.url.clone(), &processed)
            .await?;

        let is_custom = processed.link == data.url;
        let feed_entry_items = processed.entries.into_iter().map(Into::into).collect();

        let id = self
            .feed_repository
            .upsert(FeedUpsertParams {
                source_url: data.url,
                link: processed.link,
                title: processed.title,
                description: processed.description,
                refresh_interval_min,
                is_custom,
                feed_entry_items,
            })
            .await?;

        self.get_feed(id).await
    }

    async fn calculate_refresh_interval(
        &self,
        source_url: Url,
        processed: &ProcessedFeed,
    ) -> Result<u32, Error> {
        match self.feed_repository.find_by_source_url(source_url).await? {
            Some(feed) => {
                let latest_entries = self
                    .feed_entry_repository
                    .find(FeedEntryFindParams {
                        limit: Some(1),
                        feed_id: Some(feed.id),
                        ..Default::default()
                    })
                    .await?;

                let has_new = processed.entries.first().is_some_and(|a| {
                    latest_entries
                        .first()
                        .is_some_and(|b| a.published.gt(&b.published_at))
                });

                if has_new {
                    let mut dates = processed
                        .entries
                        .iter()
                        .take(SAMPLE_SIZE)
                        .map(|e| e.published)
                        .collect::<Vec<_>>();

                    if dates.len() < SAMPLE_SIZE {
                        let entries = self
                            .feed_entry_repository
                            .find(FeedEntryFindParams {
                                feed_id: Some(feed.id),
                                limit: Some(SAMPLE_SIZE - dates.len()),
                                ..Default::default()
                            })
                            .await?;

                        for entry in entries {
                            dates.push(entry.published_at);
                        }
                    }

                    if dates.len() < 2 {
                        return Ok(DEFAULT_INTERVAL);
                    }

                    let mut deltas = dates
                        .windows(2)
                        .map(|e| (e[0] - e[1]).num_minutes() as u32)
                        .collect::<Vec<_>>();

                    deltas.sort_unstable();

                    let median = if deltas.len() % 2 == 0 {
                        let right = deltas.len() / 2;
                        let left = right - 1;
                        (deltas[left] + deltas[right]) / 2
                    } else {
                        deltas[deltas.len() / 2]
                    };

                    Ok(median.clamp(MIN_INTERVAL, MAX_INTERVAL))
                } else {
                    Ok(cmp::min(
                        (feed.refresh_interval_min as f32 * BACKOFF_MULTIPLIER) as u32,
                        MAX_INTERVAL,
                    ))
                }
            }
            None => Ok(DEFAULT_INTERVAL),
        }
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
