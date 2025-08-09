use serde_json::Value;
use uuid::Uuid;

use super::JobRepository;
use crate::{Handler, RepositoryError, job::JobInsertParams};

#[derive(Debug, Clone)]
pub struct CreateJobCommand {
    pub job_type: String,
    pub data: Value,
    pub group_identifier: Option<String>,
}

pub struct CreateJobHandler {
    job_repository: Box<dyn JobRepository>,
}

impl CreateJobHandler {
    pub fn new(job_repository: impl JobRepository) -> Self {
        Self {
            job_repository: Box::new(job_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<CreateJobCommand> for CreateJobHandler {
    type Response = Uuid;
    type Error = CreateJobError;

    async fn handle(&self, data: CreateJobCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .job_repository
            .insert(JobInsertParams {
                job_type: data.job_type,
                data: data.data,
                group_identifier: data.group_identifier,
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
