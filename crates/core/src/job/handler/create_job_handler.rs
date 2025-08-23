use serde_json::Value;

use crate::{
    Handler,
    common::RepositoryError,
    job::{JobId, JobInsertParams, JobRepository},
};

#[derive(Debug, Clone)]
pub struct CreateJobCommand {
    pub job_type: String,
    pub data: Value,
    pub group_identifier: Option<String>,
}

pub struct CreateJobHandler<JR: JobRepository> {
    job_repository: JR,
}

impl<JR: JobRepository> CreateJobHandler<JR> {
    pub fn new(job_repository: JR) -> Self {
        Self { job_repository }
    }
}

#[async_trait::async_trait]
impl<JR: JobRepository> Handler<CreateJobCommand> for CreateJobHandler<JR> {
    type Response = JobId;
    type Error = CreateJobError;

    async fn handle(&self, cmd: CreateJobCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .job_repository
            .insert(JobInsertParams {
                job_type: cmd.job_type,
                data: cmd.data,
                group_identifier: cmd.group_identifier,
            })
            .await?;

        Ok(id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateJobError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
