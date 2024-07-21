use std::sync::Arc;

use chrono::Utc;
use colette_core::{
    feeds::{FeedCreateData, FeedsRepository, ProcessedFeed},
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

    async fn refresh(&self, feed_id: i64, mut url: String) {
        println!("{}: refreshing {}", Utc::now().to_rfc3339(), url);

        let feed = self.scraper.scrape(&mut url).unwrap();

        let mut profiles_stream = self.profiles_repo.iterate(feed_id);

        while let Some(Ok(profile_id)) = profiles_stream.next().await {
            let data = FeedCreateData {
                url: url.clone(),
                feed: feed.clone(),
                profile_id,
            };
            self.feeds_repo.create(data).await.unwrap();
        }
    }
}

#[async_trait::async_trait]
impl Task for RefreshTask {
    async fn run(&self) -> Result<(), task::Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self.feeds_repo.iterate();

        let tasks = feeds_stream
            .map(|item| {
                let semaphore = semaphore.clone();

                async move {
                    let _ = semaphore.acquire().await.unwrap();

                    if let Ok((feed_id, url)) = item {
                        self.refresh(feed_id, url).await
                    }
                }
            })
            .buffer_unordered(5);

        tasks.for_each(|_| async {}).await;

        Ok(())
    }
}
