use chrono::Utc;
use colette_queue::Job;
use cron::Schedule;
use serde_json::Value;
use tower::{Service as _, ServiceExt as _, util::BoxService};

use crate::job::Error;

pub struct CronWorker {
    name: String,
    schedule: Schedule,
    handler: BoxService<Job, (), Error>,
}

impl CronWorker {
    pub fn new<S: Into<String>>(
        name: S,
        schedule: Schedule,
        handler: BoxService<Job, (), Error>,
    ) -> Self {
        Self {
            name: name.into(),
            schedule,
            handler,
        }
    }

    pub async fn start(&mut self) {
        loop {
            let upcoming = self.schedule.upcoming(Utc).take(1).next().unwrap();

            let duration = (upcoming - Utc::now()).to_std().unwrap();

            tokio::time::sleep(duration).await;

            let job = match Job::create(self.name.clone(), Value::Null) {
                Ok(job) => job,
                Err(_) => {
                    continue;
                }
            };

            if let Err(e) = self.handler.ready().await.unwrap().call(job).await {
                tracing::error!("{}", e);
            };
        }
    }
}
