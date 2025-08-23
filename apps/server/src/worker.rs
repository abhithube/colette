use chrono::Utc;
use colette_job::Error;
use colette_queue::{Job, JobConsumer, TokioJobConsumer};
use cron::Schedule;
use serde_json::Value;
use tower::{Service, ServiceExt, util::BoxService};

pub struct JobWorker {
    job_consumer: TokioJobConsumer,
    job_handler: BoxService<Job, (), Error>,
}

impl JobWorker {
    pub fn new(job_consumer: TokioJobConsumer, job_handler: BoxService<Job, (), Error>) -> Self {
        Self {
            job_consumer,
            job_handler,
        }
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        while let Some(job) = self.job_consumer.pop().await? {
            if let Err(e) = self.job_handler.ready().await?.call(job).await {
                tracing::error!("{e}");
            }
        }

        Ok(())
    }
}

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
