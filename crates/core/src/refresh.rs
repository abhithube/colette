use std::sync::Arc;

use chrono::{DateTime, Local, Utc};
use colette_scraper::{FeedScraper, ProcessedFeed};
use futures::StreamExt;
use tokio::sync::Semaphore;
use url::Url;

use crate::{
    feed::{FeedCreateData, FeedRepository},
    profile::ProfileRepository,
};

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
    profile_repository: Arc<dyn ProfileRepository>,
}

impl RefreshService {
    pub fn new(
        feed_scraper: Arc<dyn FeedScraper>,
        feed_repository: Arc<dyn FeedRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            feed_scraper,
            feed_repository,
            profile_repository,
        }
    }

    pub async fn refresh_feeds(&self) -> Result<(), Error> {
        let start = Local::now();
        println!("Started refresh task at: {}", start.to_rfc3339());

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

                    if let Ok((feed_id, url)) = item {
                        let parsed = Url::parse(&url).map_err(|e| Error::Unknown(e.into()))?;

                        println!("{}: refreshing {}", Local::now().to_rfc3339(), url);

                        self.refresh_feed(feed_id, parsed).await?;
                    }

                    Ok(()) as Result<(), Error>
                }
            })
            .buffer_unordered(5);

        tasks.for_each(|_| async {}).await;

        let elasped = (Local::now().time() - start.time()).num_milliseconds();
        println!("Finished refresh task in {} ms", elasped);

        Ok(())
    }

    async fn refresh_feed(&self, feed_id: i32, mut url: Url) -> Result<(), Error> {
        let url_raw = url.to_string();

        let feed_scraper = self.feed_scraper.clone();
        let feed = tokio::task::spawn(async move { feed_scraper.scrape(&mut url) })
            .await
            .map_err(|e| Error::Unknown(e.into()))??;

        let mut profile_stream = self
            .profile_repository
            .stream(feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        while let Some(Ok(profile_id)) = profile_stream.next().await {
            self.feed_repository
                .create(FeedCreateData {
                    url: url_raw.clone(),
                    feed: Some(feed.clone()),
                    pinned: false,
                    tags: None,
                    profile_id,
                })
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        Ok(())
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
