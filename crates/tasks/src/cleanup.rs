use std::{str::FromStr, sync::Arc};

use chrono::Local;
use colette_core::feed::FeedRepository;
use cron::Schedule;

pub fn handle_cleanup_task(cron: &str, repository: Arc<dyn FeedRepository>) {
    let schedule = Schedule::from_str(cron).unwrap();

    tokio::spawn(async move {
        let repository = repository.clone();

        loop {
            let upcoming = schedule.upcoming(Local).take(1).next().unwrap();
            let duration = (upcoming - Local::now()).to_std().unwrap();

            tokio::time::sleep(duration).await;

            let start = Local::now();
            println!("Started cleanup task at: {}", start);

            match repository.cleanup_feeds().await {
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
