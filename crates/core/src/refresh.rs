use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_scraper::{FeedScraper, ProcessedFeed};
use futures::StreamExt;
use tokio::sync::Semaphore;
use url::Url;

use crate::feed::FeedRepository;

#[derive(Clone, Debug, Default)]
#[allow(dead_code)]
pub struct RefreshJob(DateTime<Utc>);
impl From<DateTime<Utc>> for RefreshJob {
    fn from(value: DateTime<Utc>) -> Self {
        RefreshJob(value)
    }
}

pub struct RefreshService {
    feed_scraper: Arc<dyn FeedScraper>,
    feed_repository: Arc<dyn FeedRepository>,
    refresh_repository: Arc<dyn RefreshRepository>,
}

impl RefreshService {
    pub fn new(
        feed_scraper: Arc<dyn FeedScraper>,
        feed_repository: Arc<dyn FeedRepository>,
        refresh_repository: Arc<dyn RefreshRepository>,
    ) -> Self {
        Self {
            feed_scraper,
            feed_repository,
            refresh_repository,
        }
    }

    pub async fn refresh_feeds(&self) -> Result<(), Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self
            .feed_repository
            .stream()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tasks = feeds_stream
            .map(|item| {
                let semaphore = semaphore.clone();

                async move {
                    let _ = semaphore
                        .acquire()
                        .await
                        .map_err(|e| Error::Unknown(e.into()))?;

                    if let Ok(url) = item {
                        let parsed = Url::parse(&url).map_err(|e| Error::Unknown(e.into()))?;

                        self.refresh_feed(parsed).await?;
                    }

                    Ok(()) as Result<(), Error>
                }
            })
            .buffer_unordered(5);

        tasks.for_each(|_| async {}).await;

        Ok(())
    }

    async fn refresh_feed(&self, mut url: Url) -> Result<(), Error> {
        let url_raw = url.to_string();

        let feed_scraper = self.feed_scraper.clone();
        let feed = tokio::task::spawn(async move { feed_scraper.scrape(&mut url) })
            .await
            .map_err(|e| Error::Unknown(e.into()))??;

        self.refresh_repository
            .refresh_feed(FeedRefreshData { url: url_raw, feed })
            .await
    }
}

#[async_trait::async_trait]
pub trait RefreshRepository: Send + Sync {
    async fn refresh_feed(&self, data: FeedRefreshData) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct FeedRefreshData {
    pub url: String,
    pub feed: ProcessedFeed,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
