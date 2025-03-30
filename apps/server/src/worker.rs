use std::sync::Arc;

use chrono::Utc;
use colette_core::job::{self, Job, JobService, JobStatus, JobUpdate};
use colette_job::Error;
use colette_queue::JobConsumer;
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

            self.handler.ready().await?;

            match self.handler.call(job).await {
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
