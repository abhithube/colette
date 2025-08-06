use serde_json::Value;
use uuid::Uuid;

use super::Error;
use crate::job::{Job, JobStatus};

#[async_trait::async_trait]
pub trait JobRepository: Send + Sync + 'static {
    async fn find(&self, params: JobFindParams) -> Result<Vec<Job>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<JobById>, Error>;

    async fn insert(&self, params: JobInsertParams) -> Result<Uuid, Error>;

    async fn update(&self, params: JobUpdateParams) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct JobFindParams {
    pub id: Option<Uuid>,
    pub group_identifier: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobById {
    pub id: Uuid,
    pub status: JobStatus,
}

#[derive(Debug, Clone)]
pub struct JobInsertParams {
    pub job_type: String,
    pub data: Value,
    pub group_identifier: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JobUpdateParams {
    pub id: Uuid,
    pub status: Option<JobStatus>,
    pub message: Option<Option<String>>,
}
