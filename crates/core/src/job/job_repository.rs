use serde_json::Value;
use uuid::Uuid;

use crate::{
    RepositoryError,
    job::{Job, JobStatus},
};

#[async_trait::async_trait]
pub trait JobRepository: Send + Sync + 'static {
    async fn find(&self, params: JobFindParams) -> Result<Vec<Job>, RepositoryError>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<JobById>, RepositoryError>;

    async fn insert(&self, params: JobInsertParams) -> Result<Uuid, RepositoryError>;

    async fn update(&self, params: JobUpdateParams) -> Result<(), RepositoryError>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), RepositoryError>;
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
