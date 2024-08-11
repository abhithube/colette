use std::{str::FromStr, sync::Arc};

use chrono::{Local, Utc};
use colette_core::{
    feeds::{FeedsCreateData, FeedsRepository, ProcessedFeed},
    profiles::ProfilesRepository,
    scraper::Scraper,
};
use cron::Schedule;
use futures::StreamExt;
use tokio::sync::Semaphore;
use url::Url;

pub struct RefreshTask {
    scraper: Arc<dyn Scraper<ProcessedFeed>>,
    feeds_repository: Arc<dyn FeedsRepository>,
    profiles_repository: Arc<dyn ProfilesRepository>,
}

impl RefreshTask {
    pub fn new(
        scraper: Arc<dyn Scraper<ProcessedFeed>>,
        feeds_repository: Arc<dyn FeedsRepository>,
        profiles_repository: Arc<dyn ProfilesRepository>,
    ) -> Self {
        Self {
            scraper,
            feeds_repository,
            profiles_repository,
        }
    }

    async fn refresh(&self, feed_id: i32, url: String) -> Result<(), anyhow::Error> {
        let mut parsed = Url::parse(&url).unwrap();

        println!("{}: refreshing {}", Utc::now().to_rfc3339(), url);

        let feed = self.scraper.scrape(&mut parsed)?;

        let mut profiles_stream = self.profiles_repository.stream_profiles(feed_id).await?;

        while let Some(Ok(profile)) = profiles_stream.next().await {
            self.feeds_repository
                .create_feed(FeedsCreateData {
                    url: url.clone(),
                    feed: feed.clone(),
                    profile_id: profile.id,
                })
                .await?;
        }

        Ok(())
    }

    async fn run(&self) -> Result<(), anyhow::Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self.feeds_repository.stream_feeds().await?;

        let tasks = feeds_stream
            .map(|item| {
                let semaphore = semaphore.clone();

                async move {
                    let _ = semaphore.acquire().await.unwrap();

                    if let Ok(feed) = item {
                        if let Err(e) = self.refresh(feed.id, feed.url).await {
                            println!("{}", e)
                        }
                    }
                }
            })
            .buffer_unordered(5);

        tasks.for_each(|_| async {}).await;

        Ok(())
    }
}

pub fn handle_refresh_task(
    cron: &str,
    scraper: Arc<dyn Scraper<ProcessedFeed>>,
    feeds_repo: Arc<dyn FeedsRepository>,
    profiles_repo: Arc<dyn ProfilesRepository>,
) {
    let schedule = Schedule::from_str(cron).unwrap();

    tokio::spawn(async move {
        let refresh_task = RefreshTask::new(scraper, feeds_repo, profiles_repo);

        loop {
            let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
            let duration = (upcoming - Local::now()).to_std().unwrap();

            tokio::time::sleep(duration).await;

            let start = Local::now();
            println!("Started refresh task at: {}", start);

            match refresh_task.run().await {
                Ok(_) => {
                    let elasped = (Local::now().time() - start.time()).num_milliseconds();
                    println!("Finished refresh task in {} ms", elasped);
                }
                Err(e) => {
                    println!("Failed refresh task: {}", e);
                }
            }
        }
    });
}
