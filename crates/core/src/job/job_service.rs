use std::sync::Arc;

use serde_json::Value;
use uuid::Uuid;

use super::{Error, Job, JobFindParams, JobRepository, JobStatus};
use crate::job::{JobInsertParams, JobUpdateParams};

pub struct JobService {
    job_repository: Arc<dyn JobRepository>,
}

impl JobService {
    pub fn new(job_repository: Arc<dyn JobRepository>) -> Self {
        Self { job_repository }
    }

    pub async fn get_job(&self, id: Uuid) -> Result<Job, Error> {
        let mut jobs = self
            .job_repository
            .find(JobFindParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if jobs.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(jobs.swap_remove(0))
    }

    pub async fn create_job(&self, data: JobCreate) -> Result<Uuid, Error> {
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

    pub async fn update_job(&self, id: Uuid, data: JobUpdate) -> Result<(), Error> {
        let Some(job) = self.job_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if job.status == JobStatus::Completed {
            return Err(Error::AlreadyCompleted(id));
        }

        self.job_repository
            .update(JobUpdateParams {
                id,
                status: data.status,
                message: data.message,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct JobCreate {
    pub job_type: String,
    pub data: Value,
    pub group_identifier: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JobUpdate {
    pub data: Option<Value>,
    pub status: Option<JobStatus>,
    pub message: Option<Option<String>>,
}
