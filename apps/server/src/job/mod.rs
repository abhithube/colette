use std::sync::Arc;

use chrono::Utc;
use colette_core::job::{self, Job, JobService, JobStatus, JobUpdate};
use colette_queue::JobConsumer;
use tower::{Service, ServiceExt, util::BoxService};

pub mod archive_thumbnail;
pub mod import_bookmarks;
pub mod import_feeds;
pub mod refresh_feeds;
pub mod scrape_bookmark;
pub mod scrape_feed;

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
            let job = self.service.get_job(job_id).await?;

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Job(#[from] job::Error),

    #[error(transparent)]
    Queue(#[from] colette_queue::Error),

    #[error(transparent)]
    Serialize(#[from] serde_json::Error),

    #[error("service error: {0}")]
    Service(String),
}
