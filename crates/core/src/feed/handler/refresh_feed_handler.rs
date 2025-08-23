use std::{cmp, sync::Arc};

use colette_http::HttpClient;
use colette_scraper::feed::{FeedScraper, ProcessedFeed};
use url::Url;

use crate::{
    Handler,
    common::RepositoryError,
    feed::{Feed, FeedFindParams, FeedId, FeedRepository, FeedUpsertParams},
    feed_entry::{FeedEntryFindParams, FeedEntryRepository},
};

pub const DEFAULT_INTERVAL: u32 = 60;
const MIN_INTERVAL: u32 = 5;
const MAX_INTERVAL: u32 = DEFAULT_INTERVAL * 24;
const SAMPLE_SIZE: usize = 20;
const BACKOFF_MULTIPLIER: f32 = 1.15;

#[derive(Debug, Clone)]
pub struct RefreshFeedCommand {
    pub url: Url,
}

pub struct RefreshFeedHandler<FR: FeedRepository, FER: FeedEntryRepository, HC: HttpClient> {
    feed_repository: FR,
    feed_entry_repository: FER,
    feed_scraper: Arc<FeedScraper<HC>>,
}

impl<FR: FeedRepository, FER: FeedEntryRepository, HC: HttpClient> RefreshFeedHandler<FR, FER, HC> {
    pub fn new(
        feed_repository: FR,
        feed_entry_repository: FER,
        feed_scraper: Arc<FeedScraper<HC>>,
    ) -> Self {
        Self {
            feed_repository,
            feed_entry_repository,
            feed_scraper,
        }
    }

    async fn get_feed(&self, id: FeedId) -> Result<Feed, RefreshFeedError> {
        let mut feeds = self
            .feed_repository
            .find(FeedFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if feeds.is_empty() {
            return Err(RefreshFeedError::NotFound(id));
        }

        Ok(feeds.swap_remove(0))
    }

    async fn calculate_refresh_interval(
        &self,
        source_url: Url,
        processed: &ProcessedFeed,
    ) -> Result<u32, RefreshFeedError> {
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

#[async_trait::async_trait]
impl<FR: FeedRepository, FER: FeedEntryRepository, HC: HttpClient> Handler<RefreshFeedCommand>
    for RefreshFeedHandler<FR, FER, HC>
{
    type Response = Feed;
    type Error = RefreshFeedError;

    async fn handle(&self, mut cmd: RefreshFeedCommand) -> Result<Self::Response, Self::Error> {
        let mut processed = match self.feed_scraper.scrape(&mut cmd.url).await {
            Ok(processed) => Ok(processed),
            Err(e) => {
                self.feed_repository.mark_as_failed(cmd.url.clone()).await?;

                Err(e)
            }
        }?;

        processed
            .entries
            .sort_by(|a, b| a.published.cmp(&b.published));

        let refresh_interval_min = self
            .calculate_refresh_interval(cmd.url.clone(), &processed)
            .await?;

        let is_custom = processed.link == cmd.url;
        let feed_entry_items = processed.entries.into_iter().map(Into::into).collect();

        let id = self
            .feed_repository
            .upsert(FeedUpsertParams {
                source_url: cmd.url,
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
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshFeedError {
    #[error("feed not found with ID: {0}")]
    NotFound(FeedId),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Scraper(#[from] colette_scraper::feed::FeedError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
