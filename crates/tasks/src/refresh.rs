use std::{str::FromStr, sync::Arc};

use chrono::{Local, Utc};
use colette_core::{
    feeds::{FeedsCreateData, FeedsRepository, ProcessedFeed},
    profiles::ProfilesRepository,
    utils::{
        scraper::Scraper,
        task::{self, Task},
    },
};
use cron::Schedule;
use futures::StreamExt;
use tokio::sync::Semaphore;
use url::Url;

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

    async fn refresh(&self, feed_id: i32, url: String) -> Result<(), task::Error> {
        let mut parsed = Url::parse(&url).unwrap();

        println!("{}: refreshing {}", Utc::now().to_rfc3339(), url);

        let feed = self
            .scraper
            .scrape(&mut parsed)
            .map_err(|e| task::Error(e.into()))?;

        let mut profiles_stream = self
            .profiles_repo
            .stream_profiles(feed_id)
            .await
            .map_err(|e| task::Error(e.into()))?;

        while let Some(Ok(profile)) = profiles_stream.next().await {
            self.feeds_repo
                .create_feed(FeedsCreateData {
                    url: url.clone(),
                    feed: feed.clone(),
                    profile_id: profile.id,
                })
                .await
                .map_err(|e| task::Error(e.into()))?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Task for RefreshTask {
    async fn run(&self) -> Result<(), task::Error> {
        let semaphore = Arc::new(Semaphore::new(5));

        let feeds_stream = self
            .feeds_repo
            .stream_feeds()
            .await
            .map_err(|e| task::Error(e.into()))?;

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
