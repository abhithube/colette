use uuid::Uuid;

use super::{Error, Job};

#[async_trait::async_trait]
pub trait JobRepository: Send + Sync + 'static {
    async fn find(&self, params: JobFindParams) -> Result<Vec<Job>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, Error>;

    async fn save(&self, data: &Job, upsert: bool) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct JobFindParams {
    pub id: Option<Uuid>,
    pub group_id: Option<String>,
}
