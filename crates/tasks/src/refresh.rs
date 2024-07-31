use std::sync::Arc;

use chrono::Utc;
use colette_core::{
    feeds::{FeedsCreateData, FeedsRepository, ProcessedFeed},
    profiles::ProfilesRepository,
    utils::{
        scraper::Scraper,
        task::{self, Task},
    },
};
use futures::StreamExt;
use tokio::sync::Semaphore;

pub struct RefreshTask {
    scraper: Arc<dyn Scraper<ProcessedFeed>>,
    feeds_repo: Arc<dyn FeedsRepository>,
    profiles_repo: Arc<dyn ProfilesRepository>,
}

impl RefreshTask {
    pub fn new(
        scraper: Arc<dyn Scraper<ProcessedFeed>>,
        feeds_repo: Arc<dyn FeedsRepository>,
        profiles_repo: Arc<dyn ProfilesRepository>,
    ) -> Self {
        Self {
            scraper,
            feeds_repo,
            profiles_repo,
        }
    }

    async fn refresh(&self, feed_id: i32, mut url: String) {
        println!("{}: refreshing {}", Utc::now().to_rfc3339(), url);

        let feed = self.scraper.scrape(&mut url).unwrap();

        let mut profiles_stream = self.profiles_repo.stream_profiles(feed_id);

        while let Some(Ok(profile)) = profiles_stream.next().await {
            self.feeds_repo
                .create_feed(FeedsCreateData {
                    url: url.clone(),
                    feed: feed.clone(),
                    profile_id: profile.id,
                })
                .await
                .unwrap();
        }
    }
}

#[async_trait::async_trait]
impl Task for RefreshTask {
    async fn run(&self) -> Result<(), task::Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self.feeds_repo.stream_feeds();

        let tasks = feeds_stream
            .map(|item| {
                let semaphore = semaphore.clone();

                async move {
                    let _ = semaphore.acquire().await.unwrap();

                    if let Ok(feed) = item {
                        self.refresh(feed.id, feed.url).await
                    }
                }
            })
            .buffer_unordered(5);

        tasks.for_each(|_| async {}).await;

        Ok(())
    }
}
