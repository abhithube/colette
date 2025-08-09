use serde_json::Value;
use uuid::Uuid;

use super::{JobRepository, JobStatus};
use crate::{Handler, RepositoryError, job::JobUpdateParams};

#[derive(Debug, Clone, Default)]
pub struct UpdateJobCommand {
    pub id: Uuid,
    pub data: Option<Value>,
    pub status: Option<JobStatus>,
    pub message: Option<Option<String>>,
}

pub struct UpdateJobHandler {
    job_repository: Box<dyn JobRepository>,
}

impl UpdateJobHandler {
    pub fn new(job_repository: impl JobRepository) -> Self {
        Self {
            job_repository: Box::new(job_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<UpdateJobCommand> for UpdateJobHandler {
    type Response = ();
    type Error = UpdateJobError;

    async fn handle(&self, data: UpdateJobCommand) -> Result<Self::Response, Self::Error> {
        let Some(job) = self.job_repository.find_by_id(data.id).await? else {
            return Err(UpdateJobError::NotFound(data.id));
        };
        if job.status == JobStatus::Completed {
            return Err(UpdateJobError::AlreadyCompleted(data.id));
        }

        self.job_repository
            .update(JobUpdateParams {
                id: data.id,
                status: data.status,
                message: data.message,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateJobError {
    #[error("Job not found with ID: {0}")]
    NotFound(Uuid),

    #[error("Already completed job with ID: {0}")]
    AlreadyCompleted(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
