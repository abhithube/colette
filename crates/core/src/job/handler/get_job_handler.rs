use crate::{
    Handler, RepositoryError,
    job::{Job, JobFindParams, JobId, JobRepository},
};

#[derive(Debug, Clone)]
pub struct GetJobQuery {
    pub id: JobId,
}

pub struct GetJobHandler {
    job_repository: Box<dyn JobRepository>,
}

impl GetJobHandler {
    pub fn new(job_repository: impl JobRepository) -> Self {
        Self {
            job_repository: Box::new(job_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetJobQuery> for GetJobHandler {
    type Response = Job;
    type Error = GetJobError;

    async fn handle(&self, query: GetJobQuery) -> Result<Self::Response, Self::Error> {
        let mut jobs = self
            .job_repository
            .find(JobFindParams {
                id: Some(query.id),
                ..Default::default()
            })
            .await?;
        if jobs.is_empty() {
            return Err(GetJobError::NotFound(query.id));
        }

        Ok(jobs.swap_remove(0))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetJobError {
    #[error("Job not found with ID: {0}")]
    NotFound(JobId),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
