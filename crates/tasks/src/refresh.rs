use std::{str::FromStr, sync::Arc};

use chrono::{Local, Utc};
use colette_core::{
    feed::{FeedCreateData, FeedRepository, ProcessedFeed},
    profile::ProfileRepository,
};
use colette_scraper::Scraper;
use cron::Schedule;
use futures::StreamExt;
use tokio::sync::Semaphore;
use url::Url;

pub struct RefreshTask {
    scraper: Arc<dyn Scraper<ProcessedFeed>>,
    feed_repository: Arc<dyn FeedRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
}

impl RefreshTask {
    pub fn new(
        scraper: Arc<dyn Scraper<ProcessedFeed>>,
        feed_repository: Arc<dyn FeedRepository>,
        profile_repository: Arc<dyn ProfileRepository>,
    ) -> Self {
        Self {
            scraper,
            feed_repository,
            profile_repository,
        }
    }

    async fn refresh(&self, feed_id: i32, url: String) -> Result<(), anyhow::Error> {
        let mut parsed = Url::parse(&url).unwrap();

        println!("{}: refreshing {}", Utc::now().to_rfc3339(), url);

        let feed = self.scraper.scrape(&mut parsed)?;

        let mut profiles_stream = self.profile_repository.stream(feed_id).await?;

        while let Some(Ok(profile_id)) = profiles_stream.next().await {
            self.feed_repository
                .create(FeedCreateData {
                    url: url.clone(),
                    feed: Some(feed.clone()),
                    folder_id: None,
                    profile_id,
                })
                .await?;
        }

        Ok(())
    }

    async fn run(&self) -> Result<(), anyhow::Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self.feed_repository.stream().await?;

        let tasks = feeds_stream
            .map(|item| {
                let semaphore = semaphore.clone();

                async move {
                    let _ = semaphore.acquire().await.unwrap();

                    if let Ok((feed_id, url)) = item {
                        if let Err(e) = self.refresh(feed_id, url).await {
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
    feed_repository: Arc<dyn FeedRepository>,
    profile_repository: Arc<dyn ProfileRepository>,
) {
    let schedule = Schedule::from_str(cron).unwrap();

    tokio::spawn(async move {
        let refresh_task = RefreshTask::new(scraper, feed_repository, profile_repository);

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
