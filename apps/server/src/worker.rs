use std::sync::Arc;

use chrono::Utc;
use colette_core::job::{self, Job, JobCreate, JobService, JobStatus, JobUpdate};
use colette_job::Error;
use colette_queue::JobConsumer;
use cron::Schedule;
use serde_json::Value;
use tower::{Service, ServiceExt, util::BoxService};

pub struct JobWorker {
    service: Arc<JobService>,
    consumer: Box<dyn JobConsumer>,
    handler: BoxService<Job, (), Error>,
}

impl JobWorker {
    pub fn new(
        service: Arc<JobService>,
        consumer: impl JobConsumer,
        handler: BoxService<Job, (), Error>,
    ) -> Self {
        Self {
            service,
            consumer: Box::new(consumer),
            handler,
        }
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        while let Some(job_id) = self.consumer.pop().await? {
            let job = match self
                .service
                .update_job(
                    job_id,
                    JobUpdate {
                        status: Some(JobStatus::Running),
                        ..Default::default()
                    },
                )
                .await
            {
                Ok(job) => job,
                Err(job::Error::AlreadyCompleted(_)) => {
                    continue;
                }
                Err(e) => return Err(Error::Job(e)),
            };

            match self.handler.ready().await?.call(job).await {
                Ok(_) => {
                    self.service
                        .update_job(
                            job_id,
                            JobUpdate {
                                status: Some(JobStatus::Completed),
                                completed_at: Some(Some(Utc::now())),
                                ..Default::default()
                            },
                        )
                        .await?
                }
                Err(e) => {
                    self.service
                        .update_job(
                            job_id,
                            JobUpdate {
                                status: Some(JobStatus::Failed),
                                message: Some(Some(e.to_string())),
                                ..Default::default()
                            },
                        )
                        .await?
                }
            };
        }

        Ok(())
    }
}

pub struct CronWorker {
    name: String,
    schedule: Schedule,
    service: Arc<JobService>,
    handler: BoxService<Job, (), Error>,
}

impl CronWorker {
    pub fn new<S: Into<String>>(
        name: S,
        schedule: Schedule,
        service: Arc<JobService>,
        handler: BoxService<Job, (), Error>,
    ) -> Self {
        Self {
            name: name.into(),
            schedule,
            service,
            handler,
        }
    }

    pub async fn start(&mut self) {
        loop {
            let upcoming = self.schedule.upcoming(Utc).take(1).next().unwrap();

            let duration = (upcoming - Utc::now()).to_std().unwrap();

            tokio::time::sleep(duration).await;

            let job = match self
                .service
                .create_job(JobCreate {
                    job_type: self.name.clone(),
                    data: Value::Null,
                    group_identifier: None,
                })
                .await
            {
                Ok(job) => job,
                Err(e) => {
                    tracing::error!("{}", e);
                    continue;
                }
            };

            let job_id = job.id;

            match self.handler.ready().await.unwrap().call(job).await {
                Ok(_) => {
                    if let Err(e) = self
                        .service
                        .update_job(
                            job_id,
                            JobUpdate {
                                status: Some(JobStatus::Completed),
                                completed_at: Some(Some(Utc::now())),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        tracing::error!("{}", e);
                    };
                }
                Err(e) => {
                    if let Err(e) = self
                        .service
                        .update_job(
                            job_id,
                            JobUpdate {
                                status: Some(JobStatus::Failed),
                                message: Some(Some(e.to_string())),
                                ..Default::default()
                            },
                        )
                        .await
                    {
                        tracing::error!("{}", e);
                    };
                }
            };
        }
    }
}
