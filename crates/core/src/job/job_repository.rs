use uuid::Uuid;

use super::{Error, Job};

#[async_trait::async_trait]
pub trait JobRepository: Send + Sync + 'static {
    async fn query(&self, params: JobParams) -> Result<Vec<Job>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Job>, Error> {
        Ok(self
            .query(JobParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &Job) -> Result<(), Error>;

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct JobParams {
    pub id: Option<Uuid>,
    pub group_id: Option<String>,
}
