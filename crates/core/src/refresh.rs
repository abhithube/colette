use std::sync::Arc;

use chrono::Local;
use colette_scraper::{feed::ProcessedFeed, Scraper};
use futures::StreamExt;
use tokio::sync::Semaphore;
use url::Url;

use crate::{
    feed::{FeedCreateData, FeedRepository},
    profile::ProfileRepository,
};

pub struct RefreshService {
    feed_scraper: Arc<dyn Scraper<ProcessedFeed>>,
    feed_repository: Arc<dyn FeedRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl RefreshService {
    pub fn new(
        feed_scraper: Arc<dyn Scraper<ProcessedFeed>>,
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

        Ok(())
    }

    async fn refresh_feed(&self, feed_id: i32, mut url: Url) -> Result<(), Error> {
        let feed = self.feed_scraper.scrape(&mut url).await?;

        let mut profiles_stream = self
            .profile_repository
            .stream(feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        while let Some(Ok(profile_id)) = profiles_stream.next().await {
            self.feed_repository
                .create(FeedCreateData {
                    url: url.to_string(),
                    feed: Some(feed.clone()),
                    folder_id: None,
                    profile_id,
                })
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Scraper(#[from] colette_scraper::Error),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
