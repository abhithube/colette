use serde_json::Value;

use crate::{
    Handler, RepositoryError,
    job::{JobId, JobRepository, JobStatus, JobUpdateParams},
};

#[derive(Debug, Clone)]
pub struct UpdateJobCommand {
    pub id: JobId,
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

    async fn handle(&self, cmd: UpdateJobCommand) -> Result<Self::Response, Self::Error> {
        let job = self
            .job_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| UpdateJobError::NotFound(cmd.id))?;
        if job.status == JobStatus::Completed {
            return Err(UpdateJobError::AlreadyCompleted(cmd.id));
        }

        self.job_repository
            .update(JobUpdateParams {
                id: cmd.id,
                status: cmd.status,
                message: cmd.message,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateJobError {
    #[error("Job not found with ID: {0}")]
    NotFound(JobId),

    #[error("Already completed job with ID: {0}")]
    AlreadyCompleted(JobId),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
