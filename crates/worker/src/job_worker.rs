use colette_queue::{Job, JobConsumer as _, TokioJobConsumer};
use tower::{Service as _, ServiceExt as _, util::BoxService};

use crate::job::Error;

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
