use std::{str::FromStr, sync::Arc};

use chrono::Local;
use colette_core::{
    feeds::FeedsRepository,
    utils::task::{self, Task},
};
use cron::Schedule;

pub struct CleanupTask {
    repo: Arc<dyn FeedsRepository>,
}

impl CleanupTask {
    pub fn new(repo: Arc<dyn FeedsRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl Task for CleanupTask {
    async fn run(&self) -> Result<(), task::Error> {
        self.repo
            .cleanup_feeds()
            .await
            .map_err(|e| task::Error(e.into()))?;

        Ok(())
    }
}

pub fn handle_cleanup_task(cron: &str, repo: Arc<dyn FeedsRepository>) {
    let schedule = Schedule::from_str(cron).unwrap();

    tokio::spawn(async move {
        let cleanup_task = CleanupTask::new(repo);

        loop {
            let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
            let duration = (upcoming - Local::now()).to_std().unwrap();

            tokio::time::sleep(duration).await;

            let start = Local::now();
            println!("Started cleanup task at: {}", start);

            match cleanup_task.run().await {
                Ok(_) => {
                    let elasped = (Local::now().time() - start.time()).num_milliseconds();
                    println!("Finished cleanup task in {} ms", elasped);
                }
                Err(e) => {
                    println!("Failed cleanup task: {}", e);
                }
            }
        }
    });
}
