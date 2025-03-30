use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use super::{Error, Job, JobParams, JobRepository, JobStatus};

pub struct JobService {
    job_repository: Box<dyn JobRepository>,
}

impl JobService {
    pub fn new(job_repository: impl JobRepository) -> Self {
        Self {
            job_repository: Box::new(job_repository),
        }
    }

    pub async fn get_job(&self, id: Uuid) -> Result<Job, Error> {
        let mut jobs = self
            .job_repository
            .query(JobParams {
                id: Some(id),
                ..Default::default()
            })
            .await?;
        if jobs.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(jobs.swap_remove(0))
    }

    pub async fn create_job(&self, data: JobCreate) -> Result<Job, Error> {
        let job = Job::builder()
            .job_type(data.job_type)
            .data(data.data)
            .maybe_group_identifier(data.group_identifier)
            .build();

        self.job_repository.save(&job).await?;

        Ok(job)
    }

    pub async fn update_job(&self, id: Uuid, data: JobUpdate) -> Result<Job, Error> {
        let Some(mut job) = self.job_repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if job.status == JobStatus::Completed {
            return Err(Error::AlreadyCompleted(id));
        }

        if let Some(data) = data.data {
            job.data = data;
        }
        if let Some(status) = data.status {
            job.status = status;
        }
        if let Some(message) = data.message {
            job.message = message;
        }
        if let Some(completed_at) = data.completed_at {
            job.completed_at = completed_at;
        }

        self.job_repository.save(&job).await?;

        Ok(job)
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
    pub completed_at: Option<Option<DateTime<Utc>>>,
}
